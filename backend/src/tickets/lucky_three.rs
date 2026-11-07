//! Lucky Three: a 3×3 grid. Three of a symbol wins that symbol's prize. Every
//! other symbol shows at most twice, so a ticket never shows two wins.

use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, Series};

/// What it costs in the shop: the expected value test keeps prizes below it
pub const PRICE: i64 = 35;
pub const CELLS: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Symbol {
    Gem,
    Coin,
    Clover,
    Ticket,
    Hourglass,
    /// Never three of a kind: fills the grids
    Star,
}

pub const SYMBOLS: [Symbol; 6] =
    [Symbol::Gem, Symbol::Coin, Symbol::Clover, Symbol::Ticket, Symbol::Hourglass, Symbol::Star];

/// Weights out of 10,000
const OUT_OF: u32 = 10_000;

/// Winning symbols, their chance out of 10,000 and their prize. The rest loses.
pub const WINS: [(Symbol, u32, Prize); 5] = [
    (Symbol::Gem, 50, Prize::Coins { amount: 1000 }),
    (Symbol::Coin, 600, Prize::Coins { amount: 200 }),
    (Symbol::Clover, 1500, Prize::Coins { amount: 50 }),
    (Symbol::Ticket, 800, Prize::Ticket { series: Series::LuckyThree }),
    (Symbol::Hourglass, 200, Prize::PocketRefill),
];

/// Platinum Three: the same grid for the top tier, with a richer table
pub const PLATINUM_PRICE: i64 = 500;
pub const PLATINUM_WINS: [(Symbol, u32, Prize); 5] = [
    (Symbol::Gem, 100, Prize::Coins { amount: 20_000 }),
    (Symbol::Coin, 800, Prize::Coins { amount: 1500 }),
    (Symbol::Clover, 2000, Prize::Coins { amount: 300 }),
    (Symbol::Ticket, 600, Prize::Ticket { series: Series::CashWheel }),
    (Symbol::Hourglass, 300, Prize::PocketRefill),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// Row by row
    pub cells: Vec<Symbol>,
    /// The symbol found three times, if any
    pub win: Option<Symbol>,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    draw_from(&WINS, rng)
}

pub fn draw_platinum(rng: &mut impl Rng) -> Drawn {
    draw_from(&PLATINUM_WINS, rng)
}

fn draw_from(wins: &[(Symbol, u32, Prize)], rng: &mut impl Rng) -> Drawn {
    let roll = rng.random_range(0..OUT_OF);
    let mut acc = 0;
    let win = wins.iter().find(|(_, w, _)| {
        acc += w;
        roll < acc
    });
    let layout = grid(win.map(|&(s, _, _)| s), rng);
    let prize = win.map_or(Prize::Nothing, |(_, _, p)| p.clone());
    Drawn { layout: serde_json::to_value(layout).expect("layout serializes"), prize }
}

/// A grid where `win` shows three times and every other symbol at most twice
fn grid(win: Option<Symbol>, rng: &mut impl Rng) -> Layout {
    let mut fillers: Vec<Symbol> = SYMBOLS.iter().filter(|&&s| Some(s) != win).flat_map(|&s| [s, s]).collect();
    fillers.shuffle(rng);
    let mut cells: Vec<Symbol> = win.map(|s| vec![s; 3]).unwrap_or_default();
    let missing = CELLS - cells.len();
    cells.extend(fillers.into_iter().take(missing));
    cells.shuffle(rng);
    Layout { cells, win }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use std::collections::HashMap;

    fn counts(cells: &[Symbol]) -> HashMap<Symbol, usize> {
        let mut m = HashMap::new();
        for &c in cells {
            *m.entry(c).or_default() += 1;
        }
        m
    }

    #[test]
    fn layouts_show_exactly_their_prize() {
        let mut rng = StdRng::seed_from_u64(7);
        for _ in 0..5000 {
            let d = draw(&mut rng);
            let layout: Layout = serde_json::from_value(d.layout).unwrap();
            assert_eq!(layout.cells.len(), CELLS);
            let c = counts(&layout.cells);
            let threes: Vec<_> = c.iter().filter(|(_, n)| **n >= 3).map(|(s, _)| *s).collect();
            match layout.win {
                Some(w) => {
                    assert_eq!(threes, vec![w], "only the winning symbol shows three times");
                    assert_eq!(c[&w], 3);
                    let prize = WINS.iter().find(|(s, _, _)| *s == w).unwrap().2.clone();
                    assert_eq!(d.prize, prize);
                }
                None => {
                    assert!(threes.is_empty(), "a losing grid has no three of a kind");
                    assert_eq!(d.prize, Prize::Nothing);
                }
            }
        }
    }

    #[test]
    fn star_never_wins() {
        assert!(WINS.iter().all(|(s, _, _)| *s != Symbol::Star));
        assert!(WINS.iter().map(|(_, w, _)| w).sum::<u32>() < OUT_OF);
        assert!(PLATINUM_WINS.iter().all(|(s, _, _)| *s != Symbol::Star));
        assert!(PLATINUM_WINS.iter().map(|(_, w, _)| w).sum::<u32>() < OUT_OF);
    }

    /// Bought tickets return 70–90 % of their price on average (game design)
    #[test]
    fn expected_value_below_price() {
        let p = |w: u32| w as f64 / OUT_OF as f64;
        let coins: f64 = WINS
            .iter()
            .map(|(_, w, prize)| match prize {
                Prize::Coins { amount } => p(*w) * *amount as f64,
                _ => 0.0,
            })
            .sum();
        let again: f64 =
            WINS.iter().filter(|(_, _, pr)| matches!(pr, Prize::Ticket { .. })).map(|(_, w, _)| p(*w)).sum();
        // A ticket prize is worth another draw: EV = coins + again × EV
        let ev = coins / (1.0 - again);
        let ratio = ev / PRICE as f64;
        assert!((0.7..=0.9).contains(&ratio), "expected value is {ratio:.2} of the price");
    }

    #[test]
    fn win_rate_is_close_to_the_table() {
        let mut rng = StdRng::seed_from_u64(42);
        let n = 100_000;
        let wins = (0..n).filter(|_| draw(&mut rng).prize != Prize::Nothing).count();
        let expected = WINS.iter().map(|(_, w, _)| *w).sum::<u32>() as f64 / OUT_OF as f64;
        let got = wins as f64 / n as f64;
        assert!((got - expected).abs() < 0.01, "win rate {got:.3}, table says {expected:.3}");
    }
}
