//! Achievements: earned once and kept, each with a reward. Some count
//! lifetime numbers, some catch a single moment.

use super::events::Event;
use crate::tickets::{Prize, Series};

/// What a counting achievement counts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metric {
    Scratched,
    Wins,
    QuestsDone,
    BestStreak,
    /// Themes and tools owned besides the defaults
    Cosmetics,
    Level,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Moment {
    /// One ticket paying this many coins or more
    WinAtLeast(i64),
    GoldenScratched,
    /// A Cash Wheel stopping on ×20
    WheelTwenty,
    /// A Ticket Chain holding this many tickets
    ChainOf(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Goal {
    Reach(Metric, i64),
    Moment(Moment),
}

pub struct Achievement {
    pub id: &'static str,
    pub goal: Goal,
    pub reward: Prize,
}

/// Every achievement, in the order players see them
pub fn all() -> Vec<Achievement> {
    use Goal::{Moment as At, Reach};
    use Metric::*;
    use Moment::*;
    let coins = |amount| Prize::Coins { amount };
    let golden = Prize::Ticket { series: Series::Golden };
    let a = |id, goal, reward| Achievement { id, goal, reward };
    vec![
        a("first_scratch", Reach(Scratched, 1), coins(50)),
        a("first_win", Reach(Wins, 1), coins(50)),
        a("scratch_100", Reach(Scratched, 100), coins(300)),
        a("scratch_1000", Reach(Scratched, 1000), golden.clone()),
        a("wins_50", Reach(Wins, 50), coins(300)),
        a("big_win", At(WinAtLeast(200)), coins(100)),
        a("jackpot", At(WinAtLeast(1000)), coins(500)),
        a("golden", At(GoldenScratched), coins(100)),
        a("wheel_20", At(WheelTwenty), coins(1000)),
        a("chain_3", At(ChainOf(3)), coins(200)),
        a("streak_7", Reach(BestStreak, 7), golden.clone()),
        a("streak_30", Reach(BestStreak, 30), Prize::Tickets { series: vec![Series::Golden, Series::Golden] }),
        a("quests_30", Reach(QuestsDone, 30), coins(500)),
        a("collector_5", Reach(Cosmetics, 5), coins(500)),
        a("level_10", Reach(Level, 10), golden),
    ]
}

/// Whether an event is the moment
pub fn caught(moment: Moment, event: &Event) -> bool {
    let Event::Scratched(s) = event else { return false };
    match moment {
        Moment::WinAtLeast(n) => s.coins >= n,
        Moment::GoldenScratched => s.series == Series::Golden,
        Moment::WheelTwenty => s.wheel == Some(20),
        Moment::ChainOf(n) => s.found >= n,
    }
}

#[cfg(test)]
mod tests {
    use super::super::events::Scratch;
    use super::*;

    #[test]
    fn ids_are_unique() {
        let ids: std::collections::HashSet<_> = all().iter().map(|a| a.id).collect();
        assert_eq!(ids.len(), all().len());
    }

    #[test]
    fn moments() {
        let s = |series, coins, wheel, found| Event::Scratched(Scratch { series, win: true, coins, wheel, found });
        assert!(caught(Moment::WinAtLeast(200), &s(Series::LuckyThree, 200, None, 0)));
        assert!(!caught(Moment::WinAtLeast(1000), &s(Series::LuckyThree, 999, None, 0)));
        assert!(caught(Moment::WheelTwenty, &s(Series::CashWheel, 1000, Some(20), 0)));
        assert!(!caught(Moment::WheelTwenty, &s(Series::CashWheel, 50, Some(10), 0)));
        assert!(caught(Moment::ChainOf(3), &s(Series::TicketChain, 0, None, 3)));
        assert!(caught(Moment::GoldenScratched, &s(Series::Golden, 1000, None, 0)));
        assert!(!caught(Moment::GoldenScratched, &Event::PocketTaken(2)));
    }
}
