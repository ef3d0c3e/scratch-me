//! Daily quests: three a day from a pool, drawn from the player and the day
//! (so they stay put), only among those the player's level can do.

use chrono::{Datelike, NaiveDate};
use rand::SeedableRng;
use rand::seq::IndexedRandom;

use super::events::Event;
use crate::tickets::{Prize, Series};

pub struct Quest {
    pub key: &'static str,
    pub goal: i32,
    pub reward: Prize,
    /// The level it needs: no Cash Wheel quest before the wheel is in the shop
    pub min_level: i32,
}

pub const PER_DAY: usize = 3;

pub fn pool() -> Vec<Quest> {
    use Series::*;
    let q = |key, goal, reward, min_level| Quest { key, goal, reward, min_level };
    vec![
        q("scratch_5", 5, Prize::Coins { amount: 100 }, 1),
        q("win_3", 3, Prize::Ticket { series: LuckyThree }, 1),
        q("coins_300", 300, Prize::Ticket { series: NumberMatch }, 1),
        q("pocket", 1, Prize::Coins { amount: 50 }, 1),
        q("envelope", 1, Prize::Coins { amount: 50 }, 1),
        q("buy", 1, Prize::Ticket { series: LuckyThree }, 1),
        q("shout", 1, Prize::Coins { amount: 50 }, 1),
        q("number_match", 1, Prize::Coins { amount: 120 }, 3),
        q("cash_wheel", 1, Prize::Coins { amount: 200 }, 5),
    ]
}

/// All three done
pub const BONUS: Prize = Prize::Ticket { series: Series::TicketChain };

pub fn quest(key: &str) -> Option<Quest> {
    pool().into_iter().find(|q| q.key == key)
}

/// How far an event moves a quest
pub fn counts(key: &str, event: &Event) -> i32 {
    match (key, event) {
        ("scratch_5", Event::Scratched(_)) => 1,
        ("win_3", Event::Scratched(s)) if s.win => 1,
        ("coins_300", Event::Scratched(s)) => s.coins.min(i32::MAX as i64) as i32,
        ("pocket", Event::PocketTaken(n)) if *n > 0 => 1,
        ("envelope", Event::EnvelopeOpened { .. }) => 1,
        ("buy", Event::Bought(_)) => 1,
        ("shout", Event::Shouted) => 1,
        ("number_match", Event::Scratched(s)) if s.series == Series::NumberMatch => 1,
        ("cash_wheel", Event::Scratched(s)) if s.series == Series::CashWheel => 1,
        _ => 0,
    }
}

fn rng(uid: i64, day: NaiveDate, salt: u64) -> rand::rngs::StdRng {
    let seed = (uid as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (day.num_days_from_ce() as u64) ^ salt;
    rand::rngs::StdRng::seed_from_u64(seed)
}

/// Whether a player can take a quest: its level, and no chat for guests
fn open_to(q: &Quest, level: i32, guest: bool) -> bool {
    q.min_level <= level && !(guest && q.key == "shout")
}

/// The day's three for a player at `level`
pub fn draw(uid: i64, day: NaiveDate, level: i32, guest: bool) -> Vec<&'static str> {
    let open: Vec<&'static str> = pool().into_iter().filter(|q| open_to(q, level, guest)).map(|q| q.key).collect();
    open.choose_multiple(&mut rng(uid, day, 0), PER_DAY).copied().collect()
}

/// A replacement for a rerolled quest: one the player does not have today
pub fn reroll(uid: i64, day: NaiveDate, level: i32, guest: bool, have: &[String]) -> Option<&'static str> {
    let open: Vec<&'static str> = pool()
        .into_iter()
        .filter(|q| open_to(q, level, guest) && !have.iter().any(|h| h == q.key))
        .map(|q| q.key)
        .collect();
    open.choose(&mut rng(uid, day, 0xA5A5)).copied()
}

#[cfg(test)]
mod tests {
    use super::super::events::Scratch;
    use super::*;

    fn d(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    #[test]
    fn three_a_day_steady_within_the_day_and_varied() {
        let a = draw(7, d("2026-11-06"), 1, false);
        assert_eq!(a.len(), PER_DAY);
        assert_eq!(a, draw(7, d("2026-11-06"), 1, false));
        let mut days = std::collections::HashSet::new();
        for i in 0..20 {
            days.insert(draw(7, d("2026-11-06") + chrono::Days::new(i), 1, false));
        }
        assert!(days.len() > 5, "quests change from day to day");
    }

    #[test]
    fn never_beyond_the_level() {
        for uid in 0..200 {
            let q = draw(uid, d("2026-11-06"), 1, false);
            assert!(!q.contains(&"cash_wheel") && !q.contains(&"number_match"));
        }
        assert!((0..200).any(|uid| draw(uid, d("2026-11-06"), 5, false).contains(&"cash_wheel")));
    }

    #[test]
    fn reroll_brings_a_new_one() {
        let have: Vec<String> = draw(3, d("2026-11-06"), 5, false).iter().map(|s| s.to_string()).collect();
        let new = reroll(3, d("2026-11-06"), 5, false, &have).unwrap();
        assert!(!have.iter().any(|h| h == new));
    }

    #[test]
    fn events_move_the_right_quests() {
        let s = |series, win, coins| Event::Scratched(Scratch { series, win, coins, wheel: None, found: 0 });
        assert_eq!(counts("scratch_5", &s(Series::LuckyThree, false, 0)), 1);
        assert_eq!(counts("win_3", &s(Series::LuckyThree, false, 0)), 0);
        assert_eq!(counts("coins_300", &s(Series::LuckyThree, true, 200)), 200);
        assert_eq!(counts("cash_wheel", &s(Series::CashWheel, false, 0)), 1);
        assert_eq!(counts("pocket", &Event::PocketTaken(3)), 1);
        assert_eq!(counts("buy", &Event::PocketTaken(3)), 0);
        for q in pool() {
            assert!(q.goal > 0 && quest(q.key).is_some());
        }
    }

    #[test]
    fn guests_get_no_chat_quest() {
        let day = d("2026-11-06");
        assert!((0..300).any(|uid| draw(uid, day, 1, false).contains(&"shout")));
        for uid in 0..300 {
            assert!(!draw(uid, day, 1, true).contains(&"shout"));
            let have: Vec<String> = draw(uid, day, 1, true).iter().map(|s| s.to_string()).collect();
            // Rerolled until nothing is left: never the shout
            let mut have = have;
            while let Some(k) = reroll(uid, day, 1, true, &have) {
                assert_ne!(k, "shout");
                have.push(k.to_owned());
            }
        }
    }
}
