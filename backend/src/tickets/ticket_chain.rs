//! Ticket Chain: six panels, each hiding a ticket or nothing. Every ticket
//! found is won, and one of them can be another chain.

use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, Series, weighted};

pub const PANELS: usize = 6;

/// How many tickets a chain holds, and how often
const COUNTS: [(usize, u32); 5] = [(0, 30), (1, 40), (2, 20), (3, 8), (4, 2)];
/// Which series each one is
pub const FOUND: [(Series, u32); 19] = [
    (Series::LuckyThree, 260),
    (Series::BoardWalk, 60),
    (Series::Battleship, 50),
    (Series::HotSpot, 50),
    (Series::WordBank, 40),
    (Series::Blackjack, 40),
    (Series::Fishing, 30),
    (Series::TicTacToe, 100),
    (Series::LuckyDice, 80),
    (Series::HighCard, 90),
    (Series::LuckyReels, 80),
    (Series::NumberMatch, 80),
    (Series::Derby, 50),
    (Series::Plinko, 30),
    (Series::Bingo, 45),
    (Series::CashWheel, 40),
    (Series::TreasureDig, 20),
    (Series::TicketChain, 25),
    (Series::Golden, 5),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// Left to right, top to bottom: a series, or nothing
    pub panels: Vec<Option<Series>>,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let n = *weighted(rng, &COUNTS);
    let mut panels: Vec<Option<Series>> = (0..PANELS).map(|i| (i < n).then(|| *weighted(rng, &FOUND))).collect();
    panels.shuffle(rng);
    let series: Vec<Series> = panels.iter().flatten().copied().collect();
    let prize = if series.is_empty() { Prize::Nothing } else { Prize::Tickets { series } };
    Drawn { layout: serde_json::to_value(Layout { panels }).expect("layout serializes"), prize }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn panels_show_the_prize() {
        let mut rng = StdRng::seed_from_u64(9);
        for _ in 0..3000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            assert_eq!(l.panels.len(), PANELS);
            let found: Vec<Series> = l.panels.iter().flatten().copied().collect();
            match d.prize {
                Prize::Nothing => assert!(found.is_empty()),
                Prize::Tickets { series } => assert_eq!(series, found),
                p => panic!("unexpected prize {p:?}"),
            }
        }
    }

    /// A chain must end: on average each chain holds well under one more chain
    #[test]
    fn chains_end() {
        let mean: f64 = COUNTS.iter().map(|(n, w)| *n as f64 * *w as f64).sum::<f64>() / 100.0;
        let chain_share = FOUND.iter().find(|(s, _)| *s == Series::TicketChain).unwrap().1 as f64
            / FOUND.iter().map(|(_, w)| *w as f64).sum::<f64>();
        assert!(mean * chain_share < 0.1);
    }
}
