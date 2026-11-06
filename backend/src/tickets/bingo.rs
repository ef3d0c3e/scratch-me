//! Bingo: numbers called, and your 5×5 card (free middle). Every row, column
//! or diagonal fully called is a line; more lines pay more.
//!
//! Like real tickets the outcome comes first: how many lines, then calls
//! that complete exactly those, padded with near misses.

use rand::Rng;
use rand::seq::{IndexedRandom, SliceRandom, index};
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 100;
/// Calls shown, at least
pub const CALLS: usize = 22;
const FREE: u8 = 0;

/// Lines won and how often, out of 1,000
const OUTCOMES: [(usize, u32); 5] = [(0, 700), (1, 200), (2, 70), (3, 25), (4, 5)];

pub fn pays(lines: usize) -> i64 {
    match lines {
        0 => 0,
        1 => 120,
        2 => 300,
        3 => 800,
        _ => 2500,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// Row by row; 0 is the free middle
    pub card: Vec<Vec<u8>>,
    pub calls: Vec<u8>,
    /// Lines completed, by index in `all_lines`
    pub lines: Vec<usize>,
}

/// The 12 lines as cell positions (row, column)
pub fn all_lines() -> Vec<Vec<(usize, usize)>> {
    let mut out: Vec<Vec<(usize, usize)>> = (0..5).map(|r| (0..5).map(|c| (r, c)).collect()).collect();
    out.extend((0..5).map(|c| (0..5).map(|r| (r, c)).collect()));
    out.push((0..5).map(|i| (i, i)).collect());
    out.push((0..5).map(|i| (i, 4 - i)).collect());
    out
}

pub fn completed(card: &[Vec<u8>], calls: &[u8]) -> Vec<usize> {
    all_lines()
        .iter()
        .enumerate()
        .filter(|(_, l)| l.iter().all(|&(r, c)| card[r][c] == FREE || calls.contains(&card[r][c])))
        .map(|(i, _)| i)
        .collect()
}

fn card(rng: &mut impl Rng) -> Vec<Vec<u8>> {
    // Column B holds 1–15, I 16–30, and so on, as on a real card
    let cols: Vec<Vec<u8>> =
        (0..5u8).map(|c| index::sample(rng, 15, 5).into_iter().map(|i| c * 15 + 1 + i as u8).collect()).collect();
    let mut g: Vec<Vec<u8>> = (0..5).map(|r| (0..5).map(|c| cols[c][r]).collect()).collect();
    g[2][2] = FREE;
    g
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let want = *weighted(rng, &OUTCOMES);
    loop {
        let g = card(rng);
        let mut chosen: Vec<usize> = (0..12).collect();
        chosen.shuffle(rng);
        chosen.truncate(want);
        let mut calls: Vec<u8> = vec![];
        for &i in &chosen {
            for &(r, c) in &all_lines()[i] {
                let n = g[r][c];
                if n != FREE && !calls.contains(&n) {
                    calls.push(n);
                }
            }
        }
        // Near misses first: card numbers that leave every other line one short or more
        let mut on_card: Vec<u8> = g.iter().flatten().copied().filter(|&n| n != FREE && !calls.contains(&n)).collect();
        on_card.shuffle(rng);
        for n in on_card {
            if calls.len() >= CALLS - 6 {
                break;
            }
            calls.push(n);
            if completed(&g, &calls).len() > want {
                calls.pop();
            }
        }
        // Then numbers off the card
        let off: Vec<u8> = (1..=75).filter(|n| !g.iter().flatten().any(|x| x == n) && !calls.contains(n)).collect();
        while calls.len() < CALLS {
            let n = *off.choose(rng).expect("numbers left");
            if !calls.contains(&n) {
                calls.push(n);
            }
        }
        let lines = completed(&g, &calls);
        if lines.len() != want {
            continue;
        }
        calls.shuffle(rng);
        let total = pays(want);
        let prize = if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing };
        return Drawn { layout: serde_json::to_value(Layout { card: g, calls, lines }).expect("serializes"), prize };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn cards_and_calls_show_the_prize() {
        let mut rng = StdRng::seed_from_u64(12);
        for _ in 0..2000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            assert_eq!(l.card[2][2], 0);
            for (c, col) in (0..5).map(|c| (c, (0..5).map(|r| l.card[r][c]).collect::<Vec<_>>())) {
                for n in col.iter().filter(|n| **n != 0) {
                    assert!((c as u8 * 15 + 1..=c as u8 * 15 + 15).contains(n), "column {c} holds {n}");
                }
            }
            let mut calls = l.calls.clone();
            calls.sort();
            calls.dedup();
            assert_eq!(calls.len(), l.calls.len(), "no number called twice");
            assert!(l.calls.len() >= CALLS);
            assert_eq!(completed(&l.card, &l.calls), l.lines);
            let total = pays(l.lines.len());
            assert_eq!(d.prize, if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing });
        }
    }
}
