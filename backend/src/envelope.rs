//! The daily envelope: one a calendar day in the player's time zone. Days in a
//! row grow the streak, and the contents grow with it over a 7-day cycle.

use chrono::{DateTime, Days, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use serde::Serialize;

use crate::tickets::Series;

pub const CYCLE: i32 = 7;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Contents {
    pub coins: i64,
    pub tickets: Vec<Series>,
}

/// What the envelope of the `day`-th day in a row holds (day ≥ 1): the
/// better tickets come later in the week, a Golden Ticket on the 7th
pub fn contents(day: i32) -> Contents {
    use Series::*;
    let (coins, tickets) = match (day.max(1) - 1) % CYCLE + 1 {
        1 => (100, vec![]),
        2 => (0, vec![LuckyThree, LuckyThree]),
        3 => (0, vec![NumberMatch]),
        4 => (200, vec![LuckyThree, LuckyThree]),
        5 => (0, vec![TicketChain]),
        6 => (300, vec![LuckyThree, LuckyThree, LuckyThree]),
        _ => (200, vec![Golden]),
    };
    Contents { coins, tickets }
}

/// The player's calendar day at `now`
pub fn today(tz: Tz, now: DateTime<Utc>) -> NaiveDate {
    now.with_timezone(&tz).date_naive()
}

/// When the player's next day starts
pub fn next_day_at(tz: Tz, now: DateTime<Utc>) -> DateTime<Utc> {
    let tomorrow = today(tz, now) + Days::new(1);
    let midnight = tomorrow.and_hms_opt(0, 0, 0).expect("midnight exists");
    // A DST change can skip midnight: the day then starts at 01:00
    tz.from_local_datetime(&midnight)
        .earliest()
        .or_else(|| tz.from_local_datetime(&(midnight + chrono::Duration::hours(1))).earliest())
        .map(|t| t.with_timezone(&Utc))
        .unwrap_or_else(|| now + chrono::Duration::days(1))
}

/// The streak as it stands today: kept when the last claim was today or
/// yesterday, else broken
pub fn current(day: i32, last: Option<NaiveDate>, today: NaiveDate) -> i32 {
    match last {
        Some(d) if d == today || d + Days::new(1) == today => day,
        _ => 0,
    }
}

/// Days missed since the last claim (0 when it was today or yesterday)
pub fn missed(last: Option<NaiveDate>, today: NaiveDate) -> i32 {
    last.map_or(0, |l| ((today - l).num_days() - 1).max(0) as i32)
}

/// The last claim as a streak shield sees it: the missed days covered, as if
/// claimed yesterday
pub fn shielded(today: NaiveDate) -> Option<NaiveDate> {
    today.pred_opt()
}

/// Claiming today: the new streak day, None when today is already claimed
pub fn claim(day: i32, last: Option<NaiveDate>, today: NaiveDate) -> Option<i32> {
    if last == Some(today) {
        return None;
    }
    Some(current(day, last, today) + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    #[test]
    fn streak_grows_breaks_and_refuses_twice() {
        assert_eq!(claim(0, None, d("2026-11-06")), Some(1));
        assert_eq!(claim(1, Some(d("2026-11-05")), d("2026-11-06")), Some(2));
        assert_eq!(claim(5, Some(d("2026-11-04")), d("2026-11-06")), Some(1), "a missed day resets");
        assert_eq!(claim(3, Some(d("2026-11-06")), d("2026-11-06")), None);
        assert_eq!(current(4, Some(d("2026-11-05")), d("2026-11-06")), 4);
        assert_eq!(current(4, Some(d("2026-11-01")), d("2026-11-06")), 0);
        assert_eq!(missed(Some(d("2026-11-04")), d("2026-11-06")), 1);
        assert_eq!(missed(Some(d("2026-11-05")), d("2026-11-06")), 0);
        assert_eq!(missed(None, d("2026-11-06")), 0);
        // A shield makes it as if claimed yesterday: the streak goes on
        assert_eq!(claim(5, shielded(d("2026-11-06")), d("2026-11-06")), Some(6));
    }

    #[test]
    fn contents_loop_every_week() {
        assert_eq!(contents(1), Contents { coins: 100, tickets: vec![] });
        assert_eq!(contents(7), Contents { coins: 200, tickets: vec![Series::Golden] });
        assert_eq!(contents(8), contents(1));
        assert_eq!(contents(14), contents(7));
        assert_eq!(contents(0), contents(1));
    }

    #[test]
    fn days_follow_the_players_zone() {
        // 23:30 UTC is already the next day in Paris
        let now: DateTime<Utc> = "2026-11-06T23:30:00Z".parse().unwrap();
        assert_eq!(today(chrono_tz::UTC, now), d("2026-11-06"));
        assert_eq!(today(chrono_tz::Europe::Paris, now), d("2026-11-07"));
        assert_eq!(next_day_at(chrono_tz::UTC, now), "2026-11-07T00:00:00Z".parse::<DateTime<Utc>>().unwrap());
        assert_eq!(
            next_day_at(chrono_tz::Europe::Paris, now),
            "2026-11-07T23:00:00Z".parse::<DateTime<Utc>>().unwrap()
        );
    }

    #[test]
    fn skipped_midnight() {
        // America/Santiago skips 00:00 when DST starts (2026-09-06)
        let now: DateTime<Utc> = "2026-09-05T12:00:00Z".parse().unwrap();
        let next = next_day_at(chrono_tz::America::Santiago, now);
        assert_eq!(today(chrono_tz::America::Santiago, next), d("2026-09-06"));
    }
}
