//! Fishing: four casts, each bringing up a fish (or an old boot). Fish pay by
//! rarity, and every species caught goes into the player's aquarium.

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 90;
pub const CASTS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

/// Every species, its rarity and what it pays
pub const SPECIES: [(&str, Rarity, i64); 12] = [
    ("sardine", Rarity::Common, 5),
    ("perch", Rarity::Common, 5),
    ("mackerel", Rarity::Common, 5),
    ("carp", Rarity::Common, 5),
    ("trout", Rarity::Common, 5),
    ("pike", Rarity::Uncommon, 20),
    ("salmon", Rarity::Uncommon, 20),
    ("tuna", Rarity::Uncommon, 20),
    ("swordfish", Rarity::Uncommon, 20),
    ("pufferfish", Rarity::Rare, 100),
    ("octopus", Rarity::Rare, 100),
    ("golden_koi", Rarity::Legendary, 600),
];

/// How often each rarity bites, against an old boot
const BITES: [(Option<Rarity>, u32); 5] = [
    (None, 25),
    (Some(Rarity::Common), 60),
    (Some(Rarity::Uncommon), 25),
    (Some(Rarity::Rare), 7),
    (Some(Rarity::Legendary), 1),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cast {
    /// None: a boot
    pub fish: Option<String>,
    pub coins: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub casts: Vec<Cast>,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let casts: Vec<Cast> = (0..CASTS)
        .map(|_| match *weighted(rng, &BITES) {
            None => Cast { fish: None, coins: 0 },
            Some(r) => {
                let of: Vec<_> = SPECIES.iter().filter(|(_, rr, _)| *rr == r).collect();
                let (name, _, coins) = of[rng.random_range(0..of.len())];
                Cast { fish: Some((*name).to_owned()), coins: *coins }
            }
        })
        .collect();
    let total: i64 = casts.iter().map(|c| c.coins).sum();
    let prize = if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing };
    Drawn { layout: serde_json::to_value(Layout { casts }).expect("serializes"), prize }
}

/// The species a fishing layout brought up
pub fn caught(layout: &serde_json::Value) -> Vec<String> {
    serde_json::from_value::<Layout>(layout.clone())
        .map(|l| l.casts.into_iter().filter_map(|c| c.fish).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn casts_pay_their_fish() {
        let mut rng = rand::rng();
        let mut seen = std::collections::HashSet::new();
        for _ in 0..20_000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout.clone()).unwrap();
            assert_eq!(l.casts.len(), CASTS);
            for c in &l.casts {
                match &c.fish {
                    Some(f) => {
                        let (_, _, coins) = SPECIES.iter().find(|(n, _, _)| n == f).unwrap();
                        assert_eq!(c.coins, *coins);
                        seen.insert(f.clone());
                    }
                    None => assert_eq!(c.coins, 0),
                }
            }
            assert_eq!(d.prize.coins(), l.casts.iter().map(|c| c.coins).sum::<i64>());
            assert_eq!(caught(&d.layout).len(), l.casts.iter().filter(|c| c.fish.is_some()).count());
        }
        assert_eq!(seen.len(), SPECIES.len(), "every species can bite");
    }
}
