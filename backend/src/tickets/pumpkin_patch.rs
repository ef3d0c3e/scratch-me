//! Pumpkin Patch (autumn only): nine pumpkins, each hiding some candy or a
//! ghost. The candy is counted up and pays by the total; a rare golden
//! pumpkin pays on top.

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 75;
pub const PUMPKINS: usize = 9;
/// What a pumpkin holds, and how often
const INSIDE: [(Pumpkin, u32); 5] = [
    (Pumpkin::Ghost, 30),
    (Pumpkin::Candy { candy: 1 }, 40),
    (Pumpkin::Candy { candy: 2 }, 22),
    (Pumpkin::Candy { candy: 3 }, 8),
    (Pumpkin::Golden, 1),
];
pub const GOLDEN: i64 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Pumpkin {
    Ghost,
    Candy { candy: u32 },
    Golden,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub pumpkins: Vec<Pumpkin>,
    pub candy: u32,
}

/// What a total of candy pays
pub fn pays(candy: u32) -> i64 {
    match candy {
        0..=12 => 0,
        13..=14 => 40,
        15..=16 => 150,
        17..=18 => 500,
        _ => 2500,
    }
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let pumpkins: Vec<Pumpkin> = (0..PUMPKINS).map(|_| *weighted(rng, &INSIDE)).collect();
    let candy = pumpkins.iter().map(|p| if let Pumpkin::Candy { candy } = p { *candy } else { 0 }).sum();
    let golden = pumpkins.iter().filter(|p| **p == Pumpkin::Golden).count() as i64 * GOLDEN;
    let total = pays(candy) + golden;
    let prize = if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing };
    Drawn { layout: serde_json::to_value(Layout { pumpkins, candy }).expect("serializes"), prize }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candy_is_counted_and_paid() {
        let mut rng = rand::rng();
        for _ in 0..3000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            let golden = l.pumpkins.iter().filter(|p| **p == Pumpkin::Golden).count() as i64;
            assert_eq!(d.prize.coins(), pays(l.candy) + golden * GOLDEN);
        }
    }
}
