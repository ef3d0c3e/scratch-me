//! Lucky Reels: three reels of three symbols, scratched one by one. Five
//! paylines (three rows, two diagonals); three of a kind on a line pays.
//! Mega Reels, for the top tier: four rows, eight paylines (four rows and
//! four diagonals), a richer table. Wild Reels: three rows, and a wild symbol
//! that stands in for any other on a line (three wilds pay their own prize).

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
    /// Wild Reels only: stands in for any symbol
    Wild,
}

/// How often each comes up on a reel, and what three on a line pay
pub const SYMBOLS: [(Reel, u32, i64); 5] = [
    (Reel::Cherry, 40, 60),
    (Reel::Lemon, 30, 120),
    (Reel::Bell, 17, 300),
    (Reel::Bar, 10, 1000),
    (Reel::Seven, 3, 10_000),
];

/// Mega Reels: the same odds, each line paying more
pub const MEGA_PRICE: i64 = 250;
pub const MEGA_SYMBOLS: [(Reel, u32, i64); 5] = [
    (Reel::Cherry, 40, 160),
    (Reel::Lemon, 30, 300),
    (Reel::Bell, 17, 800),
    (Reel::Bar, 10, 2500),
    (Reel::Seven, 3, 25_000),
];

/// Wild Reels: a wild now and then, every line paying a little more
pub const WILD_PRICE: i64 = 100;
pub const WILD_SYMBOLS: [(Reel, u32, i64); 6] = [
    (Reel::Cherry, 38, 40),
    (Reel::Lemon, 28, 80),
    (Reel::Bell, 16, 200),
    (Reel::Bar, 9, 650),
    (Reel::Seven, 3, 5000),
    (Reel::Wild, 6, 10_000),
];

/// Paylines as (row on reel 1, row on reel 2, row on reel 3)
pub const LINES: [[usize; 3]; 5] = [[0, 0, 0], [1, 1, 1], [2, 2, 2], [0, 1, 2], [2, 1, 0]];
pub const MEGA_LINES: [[usize; 3]; 8] =
    [[0, 0, 0], [1, 1, 1], [2, 2, 2], [3, 3, 3], [0, 1, 2], [1, 2, 3], [2, 1, 0], [3, 2, 1]];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// Reel by reel, top to bottom
    pub reels: Vec<Vec<Reel>>,
    /// The paylines that pay, by index in LINES
    pub lines: Vec<usize>,
}

fn pays_in(table: &[(Reel, u32, i64)], symbol: Reel) -> i64 {
    table.iter().find(|(s, _, _)| *s == symbol).map_or(0, |(_, _, p)| *p)
}

pub fn pays(symbol: Reel) -> i64 {
    pays_in(&SYMBOLS, symbol)
}

/// Lines that pay, and the total
pub fn score(reels: &[Vec<Reel>]) -> (Vec<usize>, i64) {
    score_on(reels, &LINES, &SYMBOLS)
}

fn score_on(reels: &[Vec<Reel>], paylines: &[[usize; 3]], table: &[(Reel, u32, i64)]) -> (Vec<usize>, i64) {
    let mut lines = vec![];
    let mut total = 0;
    for (i, [a, b, c]) in paylines.iter().enumerate() {
        let line = [reels[0][*a], reels[1][*b], reels[2][*c]];
        // The symbol the line makes: the one its non-wilds share, or wild for three wilds
        let mut tame = line.iter().copied().filter(|s| *s != Reel::Wild);
        let made = match tame.next() {
            None => Some(Reel::Wild),
            Some(first) => tame.all(|s| s == first).then_some(first),
        };
        if let Some(s) = made {
            lines.push(i);
            total += pays_in(table, s);
        }
    }
    (lines, total)
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    draw_on(3, &LINES, &SYMBOLS, rng)
}

pub fn draw_wild(rng: &mut impl Rng) -> Drawn {
    draw_on(3, &LINES, &WILD_SYMBOLS, rng)
}

pub fn draw_mega(rng: &mut impl Rng) -> Drawn {
    draw_on(4, &MEGA_LINES, &MEGA_SYMBOLS, rng)
}

fn draw_on(rows: usize, paylines: &[[usize; 3]], symbols: &[(Reel, u32, i64)], rng: &mut impl Rng) -> Drawn {
    let table: Vec<(Reel, u32)> = symbols.iter().map(|(s, w, _)| (*s, *w)).collect();
    let reels: Vec<Vec<Reel>> = (0..3).map(|_| (0..rows).map(|_| *weighted(rng, &table)).collect()).collect();
    let (lines, total) = score_on(&reels, paylines, symbols);
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
        // A wild completes any line; three wilds pay their own prize
        let wild = vec![vec![Cherry, Wild, Bar], vec![Wild, Wild, Bar], vec![Cherry, Wild, Lemon]];
        let (lines, total) = score_on(&wild, &LINES, &WILD_SYMBOLS);
        assert_eq!(lines, vec![0, 1], "cherry wild cherry, and three wilds");
        assert_eq!(total, 40 + 10_000);
        let tall = vec![vec![Bell; 4], vec![Bell; 4], vec![Bell; 4]];
        assert_eq!(score_on(&tall, &MEGA_LINES, &MEGA_SYMBOLS), ((0..8).collect(), 8 * 800));
        let d = draw_mega(&mut rand::rng());
        let l: Layout = serde_json::from_value(d.layout).unwrap();
        assert!(l.reels.iter().all(|r| r.len() == 4));
    }
}
