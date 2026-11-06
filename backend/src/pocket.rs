//! The pocket: free Lucky Three tickets that refill over time. It holds up to
//! SIZE; one comes back every REFILL while it is not full. Stored lazily as
//! (count, at): `count` held at `at`, refills counted from `at`.

use chrono::{DateTime, Duration, Utc};
use serde::Serialize;

pub const SIZE: i32 = 3;
pub const REFILL: Duration = Duration::hours(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Pocket {
    pub count: i32,
    pub size: i32,
    /// When the next ticket comes, None while full
    pub next_at: Option<DateTime<Utc>>,
}

/// The pocket as of `now`
pub fn level(stored: i32, at: DateTime<Utc>, now: DateTime<Utc>) -> Pocket {
    if stored >= SIZE {
        return Pocket { count: SIZE, size: SIZE, next_at: None };
    }
    let elapsed = (now - at).max(Duration::zero());
    let gained = (elapsed.num_seconds() / REFILL.num_seconds()) as i32;
    let count = (stored + gained).min(SIZE);
    let next_at = (count < SIZE).then(|| at + REFILL * (gained + 1));
    Pocket { count, size: SIZE, next_at }
}

/// Takes every ticket held: how many, and the new `at` to store with a count
/// of 0. A partial refill is kept, a full pocket starts over from now.
pub fn take(stored: i32, at: DateTime<Utc>, now: DateTime<Utc>) -> (i32, DateTime<Utc>) {
    let p = level(stored, at, now);
    let at = p.next_at.map_or(now, |next| next - REFILL);
    (p.count, at)
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
        assert_eq!(level(0, at, at).count, 0);
        assert_eq!(level(0, at, at).next_at, Some(at + REFILL));
        assert_eq!(level(0, at, at + REFILL - Duration::seconds(1)).count, 0);
        assert_eq!(level(0, at, at + REFILL).count, 1);
        assert_eq!(level(1, at, at + REFILL).count, 2);
        assert_eq!(level(1, at, at + REFILL).next_at, Some(at + REFILL * 2));
        let full = level(0, at, at + REFILL * 10);
        assert_eq!((full.count, full.next_at), (SIZE, None));
    }

    #[test]
    fn clock_in_the_past_or_future() {
        // A clock that went back must not take tickets away
        assert_eq!(level(2, t0(), t0() - Duration::hours(5)).count, 2);
        assert_eq!(level(SIZE + 2, t0(), t0()).count, SIZE);
    }

    #[test]
    fn taking_keeps_partial_progress() {
        let at = t0();
        // 1 held, 1.5 periods later: 2 held, half a period toward the next
        let now = at + REFILL + REFILL / 2;
        let (n, new_at) = take(1, at, now);
        assert_eq!(n, 2);
        assert_eq!(new_at, at + REFILL);
        let after = level(0, new_at, now);
        assert_eq!(after.count, 0);
        assert_eq!(after.next_at, Some(at + REFILL * 2), "the next one still comes on time");
    }

    #[test]
    fn taking_a_full_pocket_restarts_the_clock() {
        let now = t0() + Duration::days(2);
        let (n, new_at) = take(0, t0(), now);
        assert_eq!((n, new_at), (SIZE, now));
        assert_eq!(level(0, new_at, now).next_at, Some(now + REFILL));
    }

    #[test]
    fn taking_an_empty_pocket() {
        let (n, new_at) = take(0, t0(), t0() + Duration::minutes(30));
        assert_eq!((n, new_at), (0, t0()));
    }
}
