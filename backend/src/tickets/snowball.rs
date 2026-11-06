//! Snowball (winter only): a snowball rolls down five bumps of a hill. Fresh
//! snow makes it grow, a rock knocks some off; the bigger it ends, the more
//! it pays.

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 60;
pub const BUMPS: usize = 5;
const SLOPE: [(i32, u32); 4] = [(-1, 25), (1, 45), (2, 22), (3, 8)];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// What each bump does to the size
    pub bumps: Vec<i32>,
    /// The size after each bump, from 1
    pub sizes: Vec<i32>,
}

pub fn pays(size: i32) -> i64 {
    match size {
        ..=6 => 0,
        7 => 30,
        8 => 60,
        9..=10 => 150,
        11..=12 => 500,
        _ => 2500,
    }
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let bumps: Vec<i32> = (0..BUMPS).map(|_| *weighted(rng, &SLOPE)).collect();
    let mut size = 1;
    let sizes: Vec<i32> = bumps
        .iter()
        .map(|b| {
            size = (size + b).max(1);
            size
        })
        .collect();
    let total = pays(size);
    let prize = if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing };
    Drawn { layout: serde_json::to_value(Layout { bumps, sizes }).expect("serializes"), prize }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_last_size_pays() {
        let mut rng = rand::rng();
        for _ in 0..3000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            assert_eq!(l.sizes.len(), BUMPS);
            assert!(l.sizes.iter().all(|s| *s >= 1), "never smaller than a pebble");
            assert_eq!(d.prize.coins(), pays(*l.sizes.last().unwrap()));
        }
    }
}
