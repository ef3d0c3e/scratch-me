//! Quests by the hour: three at a time from a pool, drawn from the player, the
//! hour and the round (so they stay put), only among those the player's level
//! can do. Finishing a round's three opens three harder ones, up to `ROUNDS`
//! an hour; on the hour, it all starts again.

use chrono::{DateTime, DurationRound, TimeDelta, Utc};
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

/// Rounds an hour: the first, and two harder ones
pub const ROUNDS: i32 = 3;

/// The hour `now` falls in: its start
pub fn slot_of(now: DateTime<Utc>) -> DateTime<Utc> {
    now.duration_trunc(TimeDelta::hours(1)).expect("an hour fits")
}

pub fn pool() -> Vec<Quest> {
    use Series::*;
    let q = |key, goal, reward, min_level| Quest { key, goal, reward, min_level };
    vec![
        q("scratch_5", 5, Prize::Coins { amount: 100 }, 1),
        q("win_3", 3, Prize::Ticket { series: LuckyThree }, 1),
        q("coins_300", 300, Prize::Ticket { series: NumberMatch }, 1),
        q("pocket", 1, Prize::Coins { amount: 50 }, 1),
        q("buy", 1, Prize::Ticket { series: LuckyThree }, 1),
        q("shout", 1, Prize::Coins { amount: 50 }, 1),
        q("number_match", 1, Prize::Coins { amount: 120 }, 3),
        q("cash_wheel", 1, Prize::Coins { amount: 200 }, 6),
    ]
}

/// A round's three done
pub const BONUS: Prize = Prize::Ticket { series: Series::TicketChain };

/// What finishing a round's three gives: a Lucky Three, and a Ticket Chain
/// for the last round
pub fn bonus(round: i32) -> Prize {
    if round >= ROUNDS { BONUS } else { Prize::Ticket { series: Series::LuckyThree } }
}

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
        ("buy", Event::Bought(_)) => 1,
        ("shout", Event::Shouted) => 1,
        ("number_match", Event::Scratched(s)) if s.series == Series::NumberMatch => 1,
        ("cash_wheel", Event::Scratched(s)) if s.series == Series::CashWheel => 1,
        _ => 0,
    }
}

fn rng(uid: i64, slot: DateTime<Utc>, salt: u64) -> rand::rngs::StdRng {
    let hour = slot.timestamp().div_euclid(3600) as u64;
    let seed = (uid as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ hour.wrapping_mul(0x2545_F491_4F6C_DD1D) ^ salt;
    rand::rngs::StdRng::seed_from_u64(seed)
}

/// Whether a player can take a quest: its level, and no chat for guests
fn open_to(q: &Quest, level: i32, guest: bool) -> bool {
    q.min_level <= level && !(guest && q.key == "shout")
}

/// A round's three for a player at `level`
pub fn draw(uid: i64, slot: DateTime<Utc>, round: i32, level: i32, guest: bool) -> Vec<&'static str> {
    let open: Vec<&'static str> = pool().into_iter().filter(|q| open_to(q, level, guest)).map(|q| q.key).collect();
    let salt = (round.max(1) as u64 - 1).wrapping_mul(0x5851_F42D);
    open.choose_multiple(&mut rng(uid, slot, salt), PER_DAY).copied().collect()
}

/// A replacement for a rerolled quest: one the player does not have in this round
pub fn reroll(
    uid: i64,
    slot: DateTime<Utc>,
    round: i32,
    level: i32,
    guest: bool,
    have: &[String],
) -> Option<&'static str> {
    let open: Vec<&'static str> = pool()
        .into_iter()
        .filter(|q| open_to(q, level, guest) && !have.iter().any(|h| h == q.key))
        .map(|q| q.key)
        .collect();
    open.choose(&mut rng(uid, slot, 0xA5A5 ^ (round.max(1) as u64 - 1))).copied()
}

#[cfg(test)]
mod tests {
    use super::super::events::Scratch;
    use super::*;

    fn d(s: &str) -> DateTime<Utc> {
        slot_of(format!("{s}T10:20:00Z").parse().unwrap())
    }

    #[test]
    fn three_an_hour_steady_within_the_hour_and_varied() {
        let a = draw(7, d("2026-11-06"), 1, 1, false);
        assert_eq!(a.len(), PER_DAY);
        assert_eq!(a, draw(7, d("2026-11-06"), 1, 1, false));
        let mut days = std::collections::HashSet::new();
        for i in 0..20 {
            days.insert(draw(7, d("2026-11-06") + TimeDelta::hours(i), 1, 1, false));
        }
        assert!(days.len() > 5, "quests change from hour to hour");
    }

    #[test]
    fn never_beyond_the_level() {
        for uid in 0..200 {
            let q = draw(uid, d("2026-11-06"), 1, 1, false);
            assert!(!q.contains(&"cash_wheel") && !q.contains(&"number_match"));
        }
        assert!((0..200).any(|uid| draw(uid, d("2026-11-06"), 1, 6, false).contains(&"cash_wheel")));
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
        assert_eq!(goals("scratch_5"), [5, 8, 10]);
        assert_eq!(goals("coins_300"), [300, 450, 600]);
        assert_eq!(goals("pocket"), [1, 2, 2]);
        let scratch = quest("scratch_5").unwrap();
        assert_eq!(scratch.reward_in(1), Prize::Coins { amount: 100 });
        assert_eq!(scratch.reward_in(3), Prize::Coins { amount: 200 });
        assert_eq!(quest("win_3").unwrap().reward_in(3), Prize::Ticket { series: Series::LuckyThree });
        assert_eq!(bonus(1), Prize::Ticket { series: Series::LuckyThree });
        assert_eq!(bonus(ROUNDS), Prize::Ticket { series: Series::TicketChain }, "the hour's last round");
    }

    #[test]
    fn rounds_and_hours_are_new_draws() {
        let hour = d("2026-11-06");
        assert!((0..50).any(|uid| draw(uid, hour, 1, 1, false) != draw(uid, hour, 2, 1, false)));
        for uid in 0..300 {
            for round in 1..=ROUNDS {
                assert_eq!(draw(uid, hour, round, 1, false).len(), PER_DAY);
            }
        }
        assert!(quest("envelope").is_none(), "the envelope opens once a day: no hourly quest");
        let t: DateTime<Utc> = "2026-11-06T10:59:59Z".parse().unwrap();
        assert_eq!(slot_of(t), "2026-11-06T10:00:00Z".parse::<DateTime<Utc>>().unwrap());
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
