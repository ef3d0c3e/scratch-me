//! Daily quests: three at a time from a pool, drawn from the player, the day
//! and the round (so they stay put), only among those the player's level can
//! do. Finishing a round's three opens three harder ones, up to `ROUNDS` a day.

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

/// Quests in a round
pub const PER_DAY: usize = 3;

/// Rounds a day: the first, and three harder ones
pub const ROUNDS: i32 = 4;

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

/// A round's three done
pub const BONUS: Prize = Prize::Ticket { series: Series::TicketChain };

impl Quest {
    /// Its goal in `round`: half the first round's again each round, rounded up
    pub fn goal_in(&self, round: i32) -> i32 {
        let r = round.clamp(1, ROUNDS);
        (self.goal * (r + 1) + 1) / 2
    }

    /// Its reward in `round`: coins grow with the goal, tickets stay
    pub fn reward_in(&self, round: i32) -> Prize {
        let r = round.clamp(1, ROUNDS) as i64;
        match self.reward {
            Prize::Coins { amount } => Prize::Coins { amount: (amount * (r + 1) + 1) / 2 },
            ref other => other.clone(),
        }
    }

    /// Can come back in a later round: the envelope opens once a day
    fn repeats(&self) -> bool {
        self.key != "envelope"
    }
}

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

/// Whether a player can take a quest: its level, no chat for guests, and
/// after the first round only those that can be done again
fn open_to(q: &Quest, round: i32, level: i32, guest: bool) -> bool {
    q.min_level <= level && !(guest && q.key == "shout") && (round <= 1 || q.repeats())
}

/// A round's three for a player at `level`
pub fn draw(uid: i64, day: NaiveDate, round: i32, level: i32, guest: bool) -> Vec<&'static str> {
    let open: Vec<&'static str> =
        pool().into_iter().filter(|q| open_to(q, round, level, guest)).map(|q| q.key).collect();
    // Round 1 keeps the salt it always had: today's quests do not move
    let salt = (round.max(1) as u64 - 1).wrapping_mul(0x5851_F42D);
    open.choose_multiple(&mut rng(uid, day, salt), PER_DAY).copied().collect()
}

/// A replacement for a rerolled quest: one the player does not have in this round
pub fn reroll(uid: i64, day: NaiveDate, round: i32, level: i32, guest: bool, have: &[String]) -> Option<&'static str> {
    let open: Vec<&'static str> = pool()
        .into_iter()
        .filter(|q| open_to(q, round, level, guest) && !have.iter().any(|h| h == q.key))
        .map(|q| q.key)
        .collect();
    open.choose(&mut rng(uid, day, 0xA5A5 ^ (round.max(1) as u64 - 1))).copied()
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
        let a = draw(7, d("2026-11-06"), 1, 1, false);
        assert_eq!(a.len(), PER_DAY);
        assert_eq!(a, draw(7, d("2026-11-06"), 1, 1, false));
        let mut days = std::collections::HashSet::new();
        for i in 0..20 {
            days.insert(draw(7, d("2026-11-06") + chrono::Days::new(i), 1, 1, false));
        }
        assert!(days.len() > 5, "quests change from day to day");
    }

    #[test]
    fn never_beyond_the_level() {
        for uid in 0..200 {
            let q = draw(uid, d("2026-11-06"), 1, 1, false);
            assert!(!q.contains(&"cash_wheel") && !q.contains(&"number_match"));
        }
        assert!((0..200).any(|uid| draw(uid, d("2026-11-06"), 1, 5, false).contains(&"cash_wheel")));
    }

    #[test]
    fn reroll_brings_a_new_one() {
        let have: Vec<String> = draw(3, d("2026-11-06"), 1, 5, false).iter().map(|s| s.to_string()).collect();
        let new = reroll(3, d("2026-11-06"), 1, 5, false, &have).unwrap();
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
    fn later_rounds_ask_a_little_more() {
        let goals = |key| (1..=ROUNDS).map(|r| quest(key).unwrap().goal_in(r)).collect::<Vec<_>>();
        assert_eq!(goals("scratch_5"), [5, 8, 10, 13]);
        assert_eq!(goals("coins_300"), [300, 450, 600, 750]);
        assert_eq!(goals("pocket"), [1, 2, 2, 3]);
        let scratch = quest("scratch_5").unwrap();
        assert_eq!(scratch.reward_in(1), Prize::Coins { amount: 100 });
        assert_eq!(scratch.reward_in(4), Prize::Coins { amount: 250 });
        assert_eq!(quest("win_3").unwrap().reward_in(3), Prize::Ticket { series: Series::LuckyThree });
    }

    #[test]
    fn later_rounds_are_new_draws_without_the_envelope() {
        let day = d("2026-11-06");
        assert!((0..300).any(|uid| draw(uid, day, 1, 1, false).contains(&"envelope")));
        for uid in 0..300 {
            for round in 2..=ROUNDS {
                let q = draw(uid, day, round, 1, false);
                assert_eq!(q.len(), PER_DAY);
                assert!(!q.contains(&"envelope"));
            }
        }
        assert!((0..50).any(|uid| draw(uid, day, 1, 1, false) != draw(uid, day, 2, 1, false)));
    }

    #[test]
    fn guests_get_no_chat_quest() {
        let day = d("2026-11-06");
        assert!((0..300).any(|uid| draw(uid, day, 1, 1, false).contains(&"shout")));
        for uid in 0..300 {
            assert!(!draw(uid, day, 1, 1, true).contains(&"shout"));
            let have: Vec<String> = draw(uid, day, 1, 1, true).iter().map(|s| s.to_string()).collect();
            // Rerolled until nothing is left: never the shout
            let mut have = have;
            while let Some(k) = reroll(uid, day, 1, 1, true, &have) {
                assert_ne!(k, "shout");
                have.push(k.to_owned());
            }
        }
    }
}
