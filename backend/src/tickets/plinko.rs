//! Plinko: scratch to learn how many balls you drop (one to three). Each
//! bounces down eight rows of pegs into a prize bin. Bins and bounces are
//! drawn here: the screen only plays them out.

use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 140;
pub const ROWS: usize = 8;
/// What each of the nine bins pays, left to right
pub const BINS: [i64; ROWS + 1] = [3000, 500, 150, 60, 25, 60, 150, 500, 3000];
/// How likely each bin is, out of 1,000: the edges are rare
const BIN_WEIGHTS: [(usize, u32); ROWS + 1] =
    [(0, 1), (1, 15), (2, 80), (3, 180), (4, 448), (5, 180), (6, 80), (7, 15), (8, 1)];
const BALLS: [(usize, u32); 3] = [(1, 70), (2, 25), (3, 5)];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Drop {
    /// At each row: false bounces left, true right
    pub path: Vec<bool>,
    pub bin: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub balls: Vec<Drop>,
    pub bins: Vec<i64>,
}

/// A path that ends in `bin`: as many rights as the bin's number, in any order
fn path(bin: usize, rng: &mut impl Rng) -> Vec<bool> {
    let mut p: Vec<bool> = (0..ROWS).map(|i| i < bin).collect();
    p.shuffle(rng);
    p
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let n = *weighted(rng, &BALLS);
    let balls: Vec<Drop> = (0..n)
        .map(|_| {
            let bin = *weighted(rng, &BIN_WEIGHTS);
            Drop { path: path(bin, rng), bin }
        })
        .collect();
    let total: i64 = balls.iter().map(|b| BINS[b.bin]).sum();
    Drawn {
        layout: serde_json::to_value(Layout { balls, bins: BINS.to_vec() }).expect("serializes"),
        prize: Prize::Coins { amount: total },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn every_path_lands_in_its_bin() {
        let mut rng = StdRng::seed_from_u64(41);
        for _ in 0..2000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            assert!((1..=3).contains(&l.balls.len()));
            for b in &l.balls {
                assert_eq!(b.path.len(), ROWS);
                assert_eq!(b.path.iter().filter(|r| **r).count(), b.bin);
            }
            let total: i64 = l.balls.iter().map(|b| BINS[b.bin]).sum();
            assert_eq!(d.prize, Prize::Coins { amount: total });
        }
    }
}
