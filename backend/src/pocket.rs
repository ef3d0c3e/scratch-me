//! The pocket: free tickets that refill over time. It holds up to SIZE; one
//! comes back every REFILL while it is not full. Stored lazily as
//! (count, at): `count` held at `at`, refills counted from `at`.
//!
//! What each ticket is grows with the player's level: a shop ticket their
//! level opens, priced within `worth(level)`.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use rand::Rng;
use serde::Serialize;

use crate::progress::levels;
use crate::tickets::Series;

pub const SIZE: i32 = 3;
pub const REFILL: Duration = Duration::minutes(20);

/// The most a pocket ticket is worth (its shop price) at a level
pub fn worth(level: i32) -> i64 {
    (35 + 10 * (level.max(1) as i64 - 1)).min(300)
}

/// One pocket ticket for a player at `level`: among the shop tickets their
/// level opens (and in season), priced between half the level's worth and
/// all of it. The pricier, the likelier. Level 1 starts on the classic.
pub fn pick(level: i32, today: NaiveDate, rng: &mut impl Rng) -> Series {
    if level <= 1 {
        return Series::LuckyThree;
    }
    let top = worth(level);
    let open: Vec<(Series, i64)> = Series::ALL
        .into_iter()
        .filter(|s| s.in_shop() && levels::shop_level(*s) <= level && s.in_season(today))
        .filter_map(|s| s.price().map(|p| (s, p)))
        .filter(|(_, p)| *p <= top && *p * 2 >= top)
        .collect();
    let total: i64 = open.iter().map(|(_, p)| p).sum();
    if total == 0 {
        return Series::LuckyThree;
    }
    let mut roll = rng.random_range(0..total);
    for (s, p) in &open {
        if roll < *p {
            return *s;
        }
        roll -= p;
    }
    unreachable!("the roll falls within the total")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Pocket {
    pub count: i32,
    pub size: i32,
    /// When the next ticket comes, None while full
    pub next_at: Option<DateTime<Utc>>,
}

/// A stretch of time when the pocket refills twice as fast (an Hourglass
/// charm): from, until
pub type Fast = Option<(DateTime<Utc>, DateTime<Utc>)>;

/// Seconds of `[a, b)` inside the fast stretch
fn overlap(a: DateTime<Utc>, b: DateTime<Utc>, fast: Fast) -> i64 {
    fast.map_or(0, |(from, until)| (b.min(until) - a.max(from)).num_seconds().max(0))
}

/// Refill credit earned from `at` to `now`, in seconds: time counts double
/// inside the fast stretch
fn credit(at: DateTime<Utc>, now: DateTime<Utc>, fast: Fast) -> i64 {
    if now <= at {
        return 0;
    }
    (now - at).num_seconds() + overlap(at, now, fast)
}

/// Where the credit changes pace, after `t` (forward) or before it
fn edges(fast: Fast) -> Vec<DateTime<Utc>> {
    fast.map_or(vec![], |(from, until)| vec![from, until])
}

fn rate(t: DateTime<Utc>, forward: bool, fast: Fast) -> i64 {
    // The pace just after t going forward, just before it going back
    let probe = if forward { t } else { t - Duration::seconds(1) };
    match fast {
        Some((from, until)) if probe >= from && probe < until => 2,
        _ => 1,
    }
}

/// From `t`, how long (going forward or back) until `need` more seconds of credit
fn walk(mut t: DateTime<Utc>, mut need: i64, forward: bool, fast: Fast) -> DateTime<Utc> {
    loop {
        let r = rate(t, forward, fast);
        let next = edges(fast)
            .into_iter()
            .filter(|e| if forward { *e > t } else { *e < t })
            .min_by_key(|e| (*e - t).num_seconds().abs());
        let room = next.map(|e| (e - t).num_seconds().abs() * r);
        match room {
            Some(room) if room < need => {
                need -= room;
                t = next.expect("an edge");
            }
            _ => {
                // Rounded up going forward (never early), down going back
                let secs = if forward { (need + r - 1) / r } else { need / r };
                return if forward { t + Duration::seconds(secs) } else { t - Duration::seconds(secs) };
            }
        }
    }
}

/// The pocket as of `now`
pub fn level(stored: i32, at: DateTime<Utc>, now: DateTime<Utc>, fast: Fast) -> Pocket {
    if stored >= SIZE {
        return Pocket { count: SIZE, size: SIZE, next_at: None };
    }
    let c = credit(at, now, fast);
    let period = REFILL.num_seconds();
    let gained = (c / period) as i32;
    let count = (stored + gained).min(SIZE);
    let next_at = (count < SIZE).then(|| walk(now.max(at), (gained as i64 + 1) * period - c, true, fast));
    Pocket { count, size: SIZE, next_at }
}

/// Takes every ticket held: how many, and the new `at` to store with a count
/// of 0. A partial refill is kept, a full pocket starts over from now.
pub fn take(stored: i32, at: DateTime<Utc>, now: DateTime<Utc>, fast: Fast) -> (i32, DateTime<Utc>) {
    let p = level(stored, at, now, fast);
    if p.count >= SIZE {
        return (p.count, now);
    }
    // The credit toward the next ticket, kept: a new `at` that far back
    let left = credit(at, now, fast) - (credit(at, now, fast) / REFILL.num_seconds()) * REFILL.num_seconds();
    (p.count, walk(now, left, false, fast))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t0() -> DateTime<Utc> {
        "2026-11-06T10:00:00Z".parse().unwrap()
    }

    #[test]
    fn refills_one_every_period_up_to_size() {
        let at = t0();
        assert_eq!(level(0, at, at, None).count, 0);
        assert_eq!(level(0, at, at, None).next_at, Some(at + REFILL));
        assert_eq!(level(0, at, at + REFILL - Duration::seconds(1), None).count, 0);
        assert_eq!(level(0, at, at + REFILL, None).count, 1);
        assert_eq!(level(1, at, at + REFILL, None).count, 2);
        assert_eq!(level(1, at, at + REFILL, None).next_at, Some(at + REFILL * 2));
        let full = level(0, at, at + REFILL * 10, None);
        assert_eq!((full.count, full.next_at), (SIZE, None));
    }

    #[test]
    fn clock_in_the_past_or_future() {
        // A clock that went back must not take tickets away
        assert_eq!(level(2, t0(), t0() - Duration::hours(5), None).count, 2);
        assert_eq!(level(SIZE + 2, t0(), t0(), None).count, SIZE);
    }

    #[test]
    fn taking_keeps_partial_progress() {
        let at = t0();
        // 1 held, 1.5 periods later: 2 held, half a period toward the next
        let now = at + REFILL + REFILL / 2;
        let (n, new_at) = take(1, at, now, None);
        assert_eq!(n, 2);
        assert_eq!(new_at, at + REFILL);
        let after = level(0, new_at, now, None);
        assert_eq!(after.count, 0);
        assert_eq!(after.next_at, Some(at + REFILL * 2), "the next one still comes on time");
    }

    #[test]
    fn taking_a_full_pocket_restarts_the_clock() {
        let now = t0() + Duration::days(2);
        let (n, new_at) = take(0, t0(), now, None);
        assert_eq!((n, new_at), (SIZE, now));
        assert_eq!(level(0, new_at, now, None).next_at, Some(now + REFILL));
    }

    #[test]
    fn an_hourglass_doubles_the_pace_while_it_lasts() {
        let at = t0();
        // Fast for the first half hour: 20 minutes of refill every 10
        let fast = Some((at, at + Duration::minutes(30)));
        assert_eq!(level(0, at, at + Duration::minutes(10), fast).count, 1);
        assert_eq!(level(0, at, at + Duration::minutes(10), fast).next_at, Some(at + Duration::minutes(20)));
        // 30 fast minutes are 60 of credit: 3 tickets, the pocket full
        assert_eq!(level(0, at, at + Duration::minutes(30), fast).count, 3);
        // Taking at 25 minutes: 2 tickets and 10 minutes of credit kept
        let (n, new_at) = take(0, at, at + Duration::minutes(25), fast);
        assert_eq!(n, 2);
        let after = level(0, new_at, at + Duration::minutes(25), fast);
        assert_eq!(after.count, 0);
        // 5 fast minutes (10 of credit) then 0 more: the next is due at 30
        assert_eq!(after.next_at, Some(at + Duration::minutes(30)));
        // After the stretch, back to the usual pace
        assert_eq!(level(0, at + Duration::hours(1), at + Duration::hours(1) + Duration::minutes(10), fast).count, 0);
    }

    #[test]
    fn tickets_grow_with_the_level() {
        let day: NaiveDate = "2026-03-01".parse().unwrap();
        let mut rng = rand::rng();
        let mean = |level: i32, rng: &mut rand::rngs::ThreadRng| {
            (0..2000).map(|_| pick(level, day, rng).price().unwrap()).sum::<i64>() as f64 / 2000.0
        };
        for level in 1..=40 {
            for _ in 0..200 {
                let s = pick(level, day, &mut rng);
                let price = s.price().expect("a shop ticket");
                assert!(price <= worth(level), "level {level}: {s:?} is over its worth");
                assert!(levels::shop_level(s) <= level, "level {level}: {s:?} is not open yet");
                assert_ne!(s, Series::PotTicket);
            }
        }
        let means: Vec<f64> = [1, 3, 5, 10, 20, 30].iter().map(|l| mean(*l, &mut rng)).collect();
        assert!(means.windows(2).all(|w| w[0] <= w[1]), "worth rises with the level: {means:?}");
        assert_eq!(means[0], 35.0, "level 1: always the Lucky Three");
        assert!(means[5] >= 150.0, "{means:?}");
        assert_eq!(worth(1), 35);
        assert_eq!(worth(100), 300, "capped");
    }

    #[test]
    fn taking_an_empty_pocket() {
        let (n, new_at) = take(0, t0(), t0() + REFILL / 2, None);
        assert_eq!((n, new_at), (0, t0()));
    }
}
