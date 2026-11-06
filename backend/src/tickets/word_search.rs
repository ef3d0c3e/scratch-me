//! Word Search: an 8×8 grid of letters and four words to find, each with a
//! prize. Every word hidden in the grid (across, down or diagonally) pays.

use rand::Rng;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};

use super::words::WORDS;
use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 90;
pub const SIZE: usize = 8;
pub const TARGETS: usize = 4;
/// Chance out of 1,000 that each word is hidden
const HIDDEN: u32 = 200;
pub const LABELS: [(i64, u32); 5] = [(20, 40), (50, 30), (100, 20), (300, 8), (1000, 2)];
const DIRECTIONS: [(i32, i32); 3] = [(0, 1), (1, 0), (1, 1)];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    pub word: String,
    pub coins: i64,
    /// Where it is: the cells it covers, row * 8 + column; None when it is not in the grid
    pub at: Option<Vec<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// Row by row
    pub grid: Vec<char>,
    pub targets: Vec<Target>,
}

/// Every place a word appears in the grid
pub fn find(grid: &[char], word: &str) -> Option<Vec<usize>> {
    let w: Vec<char> = word.chars().collect();
    for r in 0..SIZE as i32 {
        for c in 0..SIZE as i32 {
            for (dr, dc) in DIRECTIONS {
                let cells: Vec<usize> = (0..w.len() as i32)
                    .map(|i| (r + dr * i, c + dc * i))
                    .take_while(|(rr, cc)| *rr < SIZE as i32 && *cc < SIZE as i32)
                    .map(|(rr, cc)| (rr * SIZE as i32 + cc) as usize)
                    .collect();
                if cells.len() == w.len() && cells.iter().zip(&w).all(|(&i, ch)| grid[i] == *ch) {
                    return Some(cells);
                }
            }
        }
    }
    None
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    'again: loop {
        let words: Vec<&str> = WORDS.choose_multiple(rng, TARGETS).copied().collect();
        let hide: Vec<bool> = (0..TARGETS).map(|_| rng.random_range(0..1000) < HIDDEN).collect();
        let mut grid: Vec<Option<char>> = vec![None; SIZE * SIZE];
        for (w, _) in words.iter().zip(&hide).filter(|(_, h)| **h) {
            let mut placed = false;
            for _ in 0..100 {
                let (dr, dc) = DIRECTIONS[rng.random_range(0..DIRECTIONS.len())];
                let len = w.len() as i32;
                let maxr = SIZE as i32 - dr * (len - 1);
                let maxc = SIZE as i32 - dc * (len - 1);
                let (r, c) = (rng.random_range(0..maxr), rng.random_range(0..maxc));
                let cells: Vec<usize> = (0..len).map(|i| ((r + dr * i) * SIZE as i32 + c + dc * i) as usize).collect();
                if cells.iter().zip(w.chars()).all(|(&i, ch)| grid[i].is_none_or(|g| g == ch)) {
                    for (&i, ch) in cells.iter().zip(w.chars()) {
                        grid[i] = Some(ch);
                    }
                    placed = true;
                    break;
                }
            }
            if !placed {
                continue 'again;
            }
        }
        let grid: Vec<char> =
            grid.into_iter().map(|g| g.unwrap_or_else(|| rng.random_range(b'A'..=b'Z') as char)).collect();
        // A word left out must not turn up by chance
        let targets: Vec<Target> = words
            .iter()
            .map(|w| Target { word: (*w).to_owned(), coins: *weighted(rng, &LABELS), at: find(&grid, w) })
            .collect();
        if targets.iter().zip(&hide).any(|(t, h)| t.at.is_some() != *h) {
            continue;
        }
        let total: i64 = targets.iter().filter(|t| t.at.is_some()).map(|t| t.coins).sum();
        let prize = if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing };
        return Drawn { layout: serde_json::to_value(Layout { grid, targets }).expect("serializes"), prize };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_found_are_the_words_paid() {
        let mut rng = rand::rng();
        for _ in 0..2000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            assert_eq!(l.grid.len(), SIZE * SIZE);
            for t in &l.targets {
                assert_eq!(find(&l.grid, &t.word).is_some(), t.at.is_some(), "{}", t.word);
            }
            assert_eq!(d.prize.coins(), l.targets.iter().filter(|t| t.at.is_some()).map(|t| t.coins).sum::<i64>());
        }
    }
}
