//! Lucky Reels: three reels of three symbols, scratched one by one. Five
//! paylines (three rows, two diagonals); three of a kind on a line pays.

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reel {
    Cherry,
    Lemon,
    Bell,
    Bar,
    Seven,
}

/// How often each comes up on a reel, and what three on a line pay
pub const SYMBOLS: [(Reel, u32, i64); 5] = [
    (Reel::Cherry, 40, 60),
    (Reel::Lemon, 30, 120),
    (Reel::Bell, 17, 300),
    (Reel::Bar, 10, 1000),
    (Reel::Seven, 3, 10_000),
];

/// Paylines as (row on reel 1, row on reel 2, row on reel 3)
pub const LINES: [[usize; 3]; 5] = [[0, 0, 0], [1, 1, 1], [2, 2, 2], [0, 1, 2], [2, 1, 0]];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// Reel by reel, top to bottom
    pub reels: Vec<Vec<Reel>>,
    /// The paylines that pay, by index in LINES
    pub lines: Vec<usize>,
}

pub fn pays(symbol: Reel) -> i64 {
    SYMBOLS.iter().find(|(s, _, _)| *s == symbol).map_or(0, |(_, _, p)| *p)
}

/// Lines that pay, and the total
pub fn score(reels: &[Vec<Reel>]) -> (Vec<usize>, i64) {
    let mut lines = vec![];
    let mut total = 0;
    for (i, [a, b, c]) in LINES.iter().enumerate() {
        let s = reels[0][*a];
        if reels[1][*b] == s && reels[2][*c] == s {
            lines.push(i);
            total += pays(s);
        }
    }
    (lines, total)
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let table: Vec<(Reel, u32)> = SYMBOLS.iter().map(|(s, w, _)| (*s, *w)).collect();
    let reels: Vec<Vec<Reel>> = (0..3).map(|_| (0..3).map(|_| *weighted(rng, &table)).collect()).collect();
    let (lines, total) = score(&reels);
    let prize = if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing };
    Drawn { layout: serde_json::to_value(Layout { reels, lines }).expect("serializes"), prize }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Reel::*;

    #[test]
    fn lines_pay_three_of_a_kind() {
        let reels = vec![vec![Cherry, Bell, Seven], vec![Cherry, Bell, Lemon], vec![Cherry, Lemon, Bell]];
        // Top row cherries; the falling diagonal: Seven, Bell, Cherry does not pay
        assert_eq!(score(&reels), (vec![0], 60));
        let all = vec![vec![Bell; 3], vec![Bell; 3], vec![Bell; 3]];
        assert_eq!(score(&all), (vec![0, 1, 2, 3, 4], 1500));
    }
}
