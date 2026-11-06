//! Live events pushed over the WebSocket to everyone: shouts, big wins,
//! achievements, announcements. Taken from AnyCardz. Events carry keys, not
//! texts: each client shows them in its own language.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::broadcast;

use crate::progress::Scratch;
use crate::tickets::{Prize, Series};

/// Wins kept for clients that connect later
const RECENT: usize = 30;

#[derive(Debug, Clone, Serialize)]
pub struct ChatEvent {
    pub id: i64,
    pub user: String,
    pub level: i32,
    /// An achievement id the author shows off
    pub title: Option<String>,
    pub body: String,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WinEvent {
    pub user: String,
    pub series: Series,
    pub prize: Prize,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatEvent {
    pub user: String,
    /// The achievement earned
    pub id: String,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnnouncementDto {
    pub id: i64,
    pub body: String,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Chat(ChatEvent),
    Win(WinEvent),
    Feat(FeatEvent),
    Announcement(AnnouncementDto),
    /// An announcement taken down
    Unannounced {
        id: i64,
    },
    /// To one player: a ticket sent to them
    Gift {
        from: String,
        series: Series,
    },
    /// To one player: a duel challenge, or what became of theirs
    Duel {
        id: i64,
        from: String,
        status: String,
    },
    /// To everyone: a day's pot was won
    Pot {
        user: String,
        amount: i64,
    },
}

/// One event on its way: to one player, or to everyone
#[derive(Debug)]
pub struct Envelope {
    pub to: Option<i64>,
    /// A shout: guests do not get it
    pub chat: bool,
    pub json: String,
}

impl Envelope {
    pub fn is_for(&self, uid: i64) -> bool {
        self.to.is_none_or(|to| to == uid)
    }
}

/// A win worth telling everyone about
pub fn notable(s: &Scratch, prize: &Prize) -> bool {
    s.coins >= 200
        || matches!(prize, Prize::Cosmetic { .. })
        || s.series == Series::Golden
        || s.found >= 3
        || s.wheel.is_some_and(|m| m >= 10)
}

/// Something that happened: a win or an achievement, for the feed
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FeedItem {
    Win(WinEvent),
    Feat(FeatEvent),
}

pub struct Hub {
    tx: broadcast::Sender<Arc<Envelope>>,
    feed: Mutex<VecDeque<FeedItem>>,
    /// Open sockets per user
    online: Mutex<HashMap<i64, usize>>,
}

impl Default for Hub {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(512);
        Hub { tx, feed: Mutex::new(VecDeque::new()), online: Mutex::new(HashMap::new()) }
    }
}

impl Hub {
    pub fn subscribe(&self) -> broadcast::Receiver<Arc<Envelope>> {
        self.tx.subscribe()
    }

    fn send_to(&self, to: Option<i64>, event: &Event) {
        // No connected client is fine: the event is simply dropped
        let json = serde_json::to_string(event).expect("events serialize");
        let chat = matches!(event, Event::Chat(_));
        let _ = self.tx.send(Arc::new(Envelope { to, chat, json }));
    }

    fn send(&self, event: &Event) {
        self.send_to(None, event);
    }

    /// An event for one player only
    pub fn notify(&self, uid: i64, event: Event) {
        self.send_to(Some(uid), &event);
    }

    pub fn pot(&self, user: &str, amount: i64) {
        self.send(&Event::Pot { user: user.to_owned(), amount });
    }

    fn remember(&self, item: FeedItem) {
        let mut feed = self.feed.lock().unwrap();
        feed.push_front(item);
        feed.truncate(RECENT);
    }

    pub fn chat(&self, event: ChatEvent) {
        self.send(&Event::Chat(event));
    }

    pub fn win(&self, event: WinEvent) {
        self.remember(FeedItem::Win(event.clone()));
        self.send(&Event::Win(event));
    }

    pub fn feat(&self, event: FeatEvent) {
        self.remember(FeedItem::Feat(event.clone()));
        self.send(&Event::Feat(event));
    }

    pub fn announce(&self, a: AnnouncementDto) {
        self.send(&Event::Announcement(a));
    }

    pub fn unannounce(&self, id: i64) {
        self.send(&Event::Unannounced { id });
    }

    /// Newest first
    pub fn feed(&self) -> Vec<FeedItem> {
        self.feed.lock().unwrap().iter().cloned().collect()
    }

    pub fn connected(&self, uid: i64) {
        *self.online.lock().unwrap().entry(uid).or_default() += 1;
    }

    pub fn disconnected(&self, uid: i64) {
        let mut online = self.online.lock().unwrap();
        if let Some(n) = online.get_mut(&uid) {
            *n -= 1;
            if *n == 0 {
                online.remove(&uid);
            }
        }
    }

    pub fn is_online(&self, uid: i64) -> bool {
        self.online.lock().unwrap().contains_key(&uid)
    }

    /// Players with a socket open
    pub fn online(&self) -> usize {
        self.online.lock().unwrap().len()
    }
}

/// Tells everyone what an action's progress and scratch earned, once committed
pub fn broadcast(hub: &Hub, user: &str, win: Option<(Series, &Scratch, &Prize)>, feats: &[crate::progress::Reward]) {
    let at = Utc::now();
    if let Some((series, s, prize)) = win
        && notable(s, prize)
    {
        hub.win(WinEvent { user: user.to_owned(), series, prize: prize.clone(), at });
    }
    for f in feats {
        hub.feat(FeatEvent { user: user.to_owned(), id: f.what.clone(), at });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(series: Series, coins: i64, wheel: Option<i64>, found: usize) -> Scratch {
        Scratch { series, win: coins > 0 || found > 0, coins, wheel, found }
    }

    #[test]
    fn only_notable_wins() {
        let coins = |amount| Prize::Coins { amount };
        assert!(!notable(&scratch(Series::LuckyThree, 50, None, 0), &coins(50)));
        assert!(notable(&scratch(Series::LuckyThree, 200, None, 0), &coins(200)));
        assert!(notable(&scratch(Series::CashWheel, 100, Some(10), 0), &coins(100)));
        assert!(notable(&scratch(Series::TicketChain, 0, None, 3), &Prize::Tickets { series: vec![] }));
        assert!(notable(&scratch(Series::Golden, 0, None, 0), &Prize::Nothing));
    }

    #[test]
    fn feed_keeps_the_latest_and_sends_everything() {
        let hub = Hub::default();
        let mut rx = hub.subscribe();
        for i in 0..40 {
            hub.feat(FeatEvent { user: format!("u{i}"), id: "first_win".into(), at: Utc::now() });
        }
        let feed = hub.feed();
        assert_eq!(feed.len(), RECENT);
        assert!(matches!(&feed[0], FeedItem::Feat(f) if f.user == "u39"));
        let first: serde_json::Value = serde_json::from_str(&rx.try_recv().unwrap().json).unwrap();
        assert_eq!(first["type"], "feat");
        assert_eq!(first["id"], "first_win");
    }

    fn hub_last(rx: &mut broadcast::Receiver<Arc<Envelope>>, send: impl FnOnce()) -> Arc<Envelope> {
        send();
        rx.try_recv().unwrap()
    }

    #[test]
    fn notes_reach_their_player_only() {
        let hub = Hub::default();
        let mut rx = hub.subscribe();
        hub.notify(7, Event::Gift { from: "ana".into(), series: Series::LuckyThree });
        let env = rx.try_recv().unwrap();
        assert!(env.is_for(7) && !env.is_for(8));
        hub.pot("bob", 500);
        assert!(rx.try_recv().unwrap().is_for(8), "the pot is news for all");
        assert!(!hub_last(&mut rx, || hub.pot("bob", 1)).chat);
        let shout = ChatEvent { id: 1, user: "ana".into(), level: 1, title: None, body: "hi".into(), at: Utc::now() };
        assert!(hub_last(&mut rx, || hub.chat(shout)).chat, "shouts are marked: guests skip them");
    }

    #[test]
    fn online_while_a_socket_is_open() {
        let hub = Hub::default();
        hub.connected(1);
        hub.connected(1);
        hub.disconnected(1);
        assert!(hub.is_online(1), "a second tab is still open");
        hub.disconnected(1);
        assert!(!hub.is_online(1));
        hub.disconnected(2);
        assert_eq!(hub.online(), 0);
    }
}
