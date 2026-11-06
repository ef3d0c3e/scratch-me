//! Pot Ticket: an entry in today's shared pot, and a small instant prize
//! under its foil. Buying it adds coins to the pot (`routes/pot.rs`); one
//! entry of the day wins the whole pot after midnight (UTC).

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 50;
/// Of the price, what goes into the pot
pub const TO_POT: i64 = 20;
/// What the house puts in every day's pot to start it
pub const SEED: i64 = 200;
const INSTANT: [(i64, u32); 3] = [(0, 70), (20, 28), (200, 2)];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub instant: i64,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let instant = *weighted(rng, &INSTANT);
    let prize = if instant > 0 { Prize::Coins { amount: instant } } else { Prize::Nothing };
    Drawn { layout: serde_json::to_value(Layout { instant }).expect("serializes"), prize }
}

/// Picks the day's winner among its entries
pub fn winner(entries: &[i64], rng: &mut impl Rng) -> Option<i64> {
    (!entries.is_empty()).then(|| entries[rng.random_range(0..entries.len())])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_entry_can_win() {
        let mut rng = rand::rng();
        assert_eq!(winner(&[], &mut rng), None);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..500 {
            seen.insert(winner(&[1, 2, 3], &mut rng).unwrap());
        }
        assert_eq!(seen.len(), 3);
    }
}
