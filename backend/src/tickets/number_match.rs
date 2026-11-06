//! Number Match: three winning numbers, eight of yours. Each of yours shows a
//! prize under it; every one that matches a winning number pays its prize.

use rand::Rng;
use rand::seq::{IndexedRandom, index};
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 150;
pub const WINNING: usize = 3;
pub const SPOTS: usize = 8;
/// Numbers run from 1 to this
pub const MAX: u32 = 30;
/// Chance out of 10,000 that a spot matches
const HIT: u32 = 1900;

/// Prizes shown under the numbers, and how often each shows
pub const LABELS: [(i64, u32); 7] = [(10, 300), (20, 250), (50, 200), (100, 130), (250, 80), (500, 35), (2000, 5)];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spot {
    pub n: u32,
    pub coins: i64,
    pub hit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub winning: Vec<u32>,
    pub spots: Vec<Spot>,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let winning: Vec<u32> = index::sample(rng, MAX as usize, WINNING).into_iter().map(|i| i as u32 + 1).collect();
    let misses: Vec<u32> = (1..=MAX).filter(|n| !winning.contains(n)).collect();
    let spots: Vec<Spot> = (0..SPOTS)
        .map(|_| {
            let coins = *weighted(rng, &LABELS);
            let hit = rng.random_range(0..10_000) < HIT;
            let n = if hit { *winning.choose(rng).unwrap() } else { *misses.choose(rng).unwrap() };
            Spot { n, coins, hit }
        })
        .collect();
    let total: i64 = spots.iter().filter(|s| s.hit).map(|s| s.coins).sum();
    let prize = if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing };
    Drawn { layout: serde_json::to_value(Layout { winning, spots }).expect("layout serializes"), prize }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn layout_shows_exactly_its_prize() {
        let mut rng = StdRng::seed_from_u64(5);
        for _ in 0..5000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            assert_eq!(l.winning.len(), WINNING);
            assert_eq!(l.spots.len(), SPOTS);
            let mut w = l.winning.clone();
            w.dedup();
            assert_eq!(w.len(), WINNING, "winning numbers differ");
            for s in &l.spots {
                assert!((1..=MAX).contains(&s.n));
                // What the player sees decides: a spot pays iff its number is a winning one
                assert_eq!(s.hit, l.winning.contains(&s.n));
            }
            let total: i64 = l.spots.iter().filter(|s| s.hit).map(|s| s.coins).sum();
            let expected = if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing };
            assert_eq!(d.prize, expected);
        }
    }
}
