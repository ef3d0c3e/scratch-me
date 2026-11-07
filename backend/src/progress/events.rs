//! What players do that progression counts

use crate::tickets::{Prize, Series};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Scratched(Scratch),
    /// Tickets taken from the pocket
    PocketTaken(i32),
    EnvelopeOpened {
        /// Days in a row, this one included
        streak: i32,
    },
    Bought(Series),
    /// A message in the shoutbox
    Shouted,
    /// A ticket bought for another player
    Gifted,
    /// A bundle bought
    BundleBought,
    /// Experience on top, from a Scholar charm
    ExtraXp(i64),
}

/// A ticket scratched, with what the achievements look at
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scratch {
    pub series: Series,
    /// Anything but nothing
    pub win: bool,
    /// Coins it paid, a duplicate's coins included
    pub coins: i64,
    /// A Cash Wheel's multiplier, when it spun
    pub wheel: Option<i64>,
    /// Tickets a Ticket Chain held
    pub found: usize,
}

impl Scratch {
    /// What a scratch was, from the ticket's series, layout and prize
    pub fn of(series: Series, layout: &serde_json::Value, prize: &Prize, duplicate_coins: i64) -> Scratch {
        let coins = prize.coins() + duplicate_coins;
        let wheel = (series == Series::CashWheel)
            .then(|| {
                let base = layout["base"].as_i64()?;
                let stop = layout["stop"].as_u64()? as usize;
                (base > 0).then(|| layout["wheel"][stop].as_i64()).flatten()
            })
            .flatten();
        let found = match prize {
            Prize::Tickets { series } if series_is_chain(layout) => series.len(),
            _ => 0,
        };
        Scratch { series, win: *prize != Prize::Nothing, coins, wheel, found }
    }
}

fn series_is_chain(layout: &serde_json::Value) -> bool {
    layout.get("panels").is_some()
}

/// Experience an event gives
pub fn xp(event: &Event) -> i64 {
    match event {
        // Every scratch counts; wins more, big wins more still
        Event::Scratched(s) => 10 + if s.win { 5 } else { 0 } + (s.coins / 20).min(50),
        Event::EnvelopeOpened { .. } => 20,
        Event::PocketTaken(_) | Event::Bought(_) | Event::Shouted | Event::Gifted | Event::BundleBought => 0,
        Event::ExtraXp(n) => *n,
    }
}

/// Experience for a quest done, and for all of the day's
pub const QUEST_XP: i64 = 15;
pub const BONUS_XP: i64 = 30;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn scratch_facts_from_the_layout() {
        let wheel = Scratch::of(
            Series::CashWheel,
            &json!({ "base": 100, "wheel": [1, 20], "stop": 1 }),
            &Prize::Coins { amount: 2000 },
            0,
        );
        assert_eq!(wheel.wheel, Some(20));
        assert!(wheel.win);
        let empty =
            Scratch::of(Series::CashWheel, &json!({ "base": 0, "wheel": [1, 20], "stop": 1 }), &Prize::Nothing, 0);
        assert_eq!(empty.wheel, None, "an empty spot does not spin");
        let chain = Scratch::of(
            Series::TicketChain,
            &json!({ "panels": ["golden", null, "lucky_three"] }),
            &Prize::Tickets { series: vec![Series::Golden, Series::LuckyThree] },
            0,
        );
        assert_eq!(chain.found, 2);
        let dup = Scratch::of(Series::Golden, &json!({}), &Prize::Tickets { series: vec![] }, 800);
        assert_eq!((dup.coins, dup.found), (800, 0), "a Golden bundle is not a chain");
    }

    #[test]
    fn xp_grows_with_the_win() {
        let s =
            |win, coins| Event::Scratched(Scratch { series: Series::LuckyThree, win, coins, wheel: None, found: 0 });
        assert_eq!(xp(&s(false, 0)), 10);
        assert_eq!(xp(&s(true, 50)), 17);
        assert_eq!(xp(&s(true, 100_000)), 65, "capped");
        assert_eq!(xp(&Event::Bought(Series::LuckyThree)), 0, "buying is no shortcut");
    }
}
