//! Achievements: earned once and kept, each with a reward. Some count
//! lifetime numbers, some catch a single moment.

use super::events::Event;
use crate::cosmetics::{Cosmetic, Theme, Tool};
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
    /// Fish species in the aquarium
    Species,
    Level,
    /// Coins won scratching, bonuses included, over a lifetime
    CoinsWon,
    /// Bundles bought
    Bundles,
    /// Year-round shop series scratched at least once
    SeriesTried,
    /// Year-round bundles bought at least once
    BundleKinds,
    /// Year-round seals won under at least once
    SealWins,
    /// The longest run of wins, and of losses
    BestWinRun,
    BestLoseRun,
    /// Hours with every quest round done
    FullHours,
    GiftsSent,
    DuelsWon,
    PotsWon,
    /// Coin charms worn at once (discount, luck, payout)
    CoinCharmsWorn,
    /// The level, while the account is a week old or less (0 after)
    LevelInFirstWeek,
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
    /// A Treasure Dig paying this many coins or more
    DigAtLeast(i64),
    /// A ticket of this series paying this many coins or more
    SeriesAtLeast(Series, i64),
    /// A ticket scratched between these hours, the player's time (from included)
    ScratchedBetween(u32, u32),
}

/// The shop series sold all year: those "every series" counts
pub fn year_round_series() -> Vec<Series> {
    Series::ALL.into_iter().filter(|s| s.in_shop() && s.season().is_none()).collect()
}

/// The bundles sold all year, and the seals they carry
pub fn year_round_bundles() -> Vec<&'static str> {
    crate::bundles::all().into_iter().filter(|b| !b.seasonal()).map(|b| b.key).collect()
}

pub fn year_round_seals() -> Vec<&'static str> {
    let mut seals: Vec<&'static str> =
        crate::bundles::all().into_iter().filter(|b| !b.seasonal()).map(|b| b.seal.key).collect();
    seals.dedup();
    seals
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

/// Every achievement, in the order players see them: roughly the order they
/// come in, spread out to level 50 (The House)
pub fn all() -> Vec<Achievement> {
    use Goal::{Moment as At, Reach};
    use Metric::*;
    use Moment::*;
    let coins = |amount| Prize::Coins { amount };
    let golden = Prize::Ticket { series: Series::Golden };
    let goldens = |n| Prize::Tickets { series: vec![Series::Golden; n] };
    let tickets = |s: Series, n| Prize::Tickets { series: vec![s; n] };
    let cosmetic = |item| Prize::Cosmetic { item };
    let a = |id, goal, reward| Achievement { id, goal, reward };
    vec![
        // The first steps
        a("first_scratch", Reach(Scratched, 1), coins(50)),
        a("first_win", Reach(Wins, 1), coins(50)),
        a("big_win", At(WinAtLeast(200)), coins(100)),
        a("golden", At(GoldenScratched), coins(100)),
        a("chain_3", At(ChainOf(3)), coins(200)),
        a("first_bundle", Reach(Bundles, 1), coins(200)),
        a("lucky_call", At(SeriesAtLeast(Series::CoinFlip, 250)), coins(250)),
        a("scratch_100", Reach(Scratched, 100), coins(300)),
        a("level_5", Reach(Level, 5), tickets(Series::TicketChain, 2)),
        a("streak_7", Reach(BestStreak, 7), golden.clone()),
        a("wins_50", Reach(Wins, 50), coins(300)),
        a("jackpot", At(WinAtLeast(1000)), coins(500)),
        a("quests_30", Reach(QuestsDone, 30), coins(500)),
        a("collector_5", Reach(Cosmetics, 5), coins(500)),
        a("admiral", At(SeriesAtLeast(Series::Battleship, 190)), coins(500)),
        a("hat_trick", Reach(BestWinRun, 3), coins(150)),
        a("night_owl", At(ScratchedBetween(2, 5)), coins(200)),
        a("early_bird", At(ScratchedBetween(5, 7)), coins(200)),
        a("charmed", Reach(CoinCharmsWorn, 3), coins(300)),
        a("hour_master", Reach(FullHours, 1), tickets(Series::TicketChain, 1)),
        a("pot_luck", Reach(PotsWon, 1), coins(500)),
        a("unlucky", Reach(BestLoseRun, 10), coins(500)),
        // Seasons and their trophies
        a("deep_digger", At(DigAtLeast(1000)), cosmetic(Cosmetic::Tool(Tool::Shovel))),
        a("pumpkin_king", At(SeriesAtLeast(Series::PumpkinPatch, 400)), cosmetic(Cosmetic::Theme(Theme::Spooky))),
        a("snow_champion", At(SeriesAtLeast(Series::Snowball, 500)), cosmetic(Cosmetic::Theme(Theme::Frost))),
        // Settling in
        a("scratch_500", Reach(Scratched, 500), tickets(Series::CashWheel, 1)),
        a("green_zero", At(SeriesAtLeast(Series::Roulette, 2500)), coins(1000)),
        a("chain_5", At(ChainOf(5)), golden.clone()),
        a("won_10k", Reach(CoinsWon, 10_000), coins(1000)),
        a("level_10", Reach(Level, 10), cosmetic(Cosmetic::Theme(Theme::Champion))),
        a("wheel_20", At(WheelTwenty), coins(1000)),
        a("scratch_1000", Reach(Scratched, 1000), golden.clone()),
        a("wins_250", Reach(Wins, 250), coins(1500)),
        a("aquarium", Reach(Species, 12), goldens(2)),
        a("quests_100", Reach(QuestsDone, 100), golden.clone()),
        a("bundles_10", Reach(Bundles, 10), coins(2000)),
        a("high_roller", At(WinAtLeast(5000)), coins(2000)),
        a("collector_10", Reach(Cosmetics, 10), coins(2000)),
        a("fast_learner", Reach(LevelInFirstWeek, 10), golden.clone()),
        a("generous", Reach(GiftsSent, 10), coins(1000)),
        a("duelist", Reach(DuelsWon, 10), coins(1500)),
        a("clockwork", Reach(FullHours, 10), golden.clone()),
        a("tried_them_all", Reach(SeriesTried, year_round_series().len() as i64), goldens(2)),
        a("every_bundle", Reach(BundleKinds, year_round_bundles().len() as i64), coins(3000)),
        a("every_seal", Reach(SealWins, year_round_seals().len() as i64), coins(5000)),
        // The long road
        a("level_20", Reach(Level, 20), tickets(Series::PlatinumThree, 2)),
        a("scratch_2500", Reach(Scratched, 2500), goldens(2)),
        a("won_100k", Reach(CoinsWon, 100_000), coins(5000)),
        a("streak_30", Reach(BestStreak, 30), goldens(2)),
        a("wins_1000", Reach(Wins, 1000), goldens(2)),
        a("quests_500", Reach(QuestsDone, 500), tickets(Series::PlatinumThree, 2)),
        a("royal", At(SeriesAtLeast(Series::PokerHand, 50_000)), coins(5000)),
        a("level_30", Reach(Level, 30), coins(10_000)),
        a("scratch_5000", Reach(Scratched, 5000), goldens(3)),
        a("whale", At(WinAtLeast(25_000)), coins(10_000)),
        a("collector_20", Reach(Cosmetics, 20), coins(5000)),
        a("level_40", Reach(Level, 40), coins(25_000)),
        a("streak_100", Reach(BestStreak, 100), goldens(3)),
        a("won_1m", Reach(CoinsWon, 1_000_000), coins(50_000)),
        a("scratch_10000", Reach(Scratched, 10_000), goldens(5)),
        a("level_50", Reach(Level, 50), Prize::of(vec![coins(100_000), goldens(5)])),
    ]
}

/// Whether an event is the moment; `hour` is the player's own hour now
pub fn caught(moment: Moment, event: &Event, hour: u32) -> bool {
    let Event::Scratched(s) = event else { return false };
    match moment {
        Moment::ScratchedBetween(from, to) => from <= hour && hour < to,
        Moment::WinAtLeast(n) => s.coins >= n,
        Moment::GoldenScratched => s.series == Series::Golden,
        Moment::WheelTwenty => s.wheel == Some(20),
        Moment::ChainOf(n) => s.found >= n,
        Moment::DigAtLeast(n) => s.series == Series::TreasureDig && s.coins >= n,
        Moment::SeriesAtLeast(series, n) => s.series == series && s.coins >= n,
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

    /// Something left to earn all the way to level 50: most achievements are
    /// not a matter of the first ten levels
    #[test]
    fn spread_to_level_fifty() {
        let all = all();
        let levels: Vec<i64> = all
            .iter()
            .filter_map(|a| match a.goal {
                Goal::Reach(Metric::Level, n) => Some(n),
                _ => None,
            })
            .collect();
        assert_eq!(levels, [5, 10, 20, 30, 40, 50]);
        let past_ten = all.iter().position(|a| a.id == "level_10").unwrap();
        assert!(past_ten * 2 < all.len(), "{past_ten} of {} come before level 10", all.len());
    }

    fn caught_at(m: Moment, e: &Event) -> bool {
        caught(m, e, 12)
    }

    #[test]
    fn the_hour_of_a_scratch() {
        let s = Event::Scratched(Scratch { series: Series::LuckyThree, win: false, coins: 0, wheel: None, found: 0 });
        assert!(caught(Moment::ScratchedBetween(2, 5), &s, 3));
        assert!(!caught(Moment::ScratchedBetween(2, 5), &s, 5));
        assert!(!caught(Moment::ScratchedBetween(5, 7), &Event::PocketTaken(1), 6), "a scratch, not anything");
        let all = year_round_series();
        assert!(
            !all.contains(&Series::PumpkinPatch) && !all.contains(&Series::PotTicket) && all.contains(&Series::Keno)
        );
        assert!(!year_round_bundles().contains(&"halloween"));
        assert!(year_round_seals().contains(&"fortune") && !year_round_seals().contains(&"winter"));
    }

    #[test]
    fn moments() {
        let s = |series, coins, wheel, found| Event::Scratched(Scratch { series, win: true, coins, wheel, found });
        assert!(caught_at(Moment::WinAtLeast(200), &s(Series::LuckyThree, 200, None, 0)));
        assert!(!caught_at(Moment::WinAtLeast(1000), &s(Series::LuckyThree, 999, None, 0)));
        assert!(caught_at(Moment::WheelTwenty, &s(Series::CashWheel, 1000, Some(20), 0)));
        assert!(!caught_at(Moment::WheelTwenty, &s(Series::CashWheel, 50, Some(10), 0)));
        assert!(caught_at(Moment::ChainOf(3), &s(Series::TicketChain, 0, None, 3)));
        assert!(caught_at(Moment::GoldenScratched, &s(Series::Golden, 1000, None, 0)));
        assert!(!caught_at(Moment::GoldenScratched, &Event::PocketTaken(2)));
    }
}
