//! Derby: six horses race. You hold two of them, each with a prize: if one
//! of yours wins the race, it pays. The whole finishing order is drawn
//! here, so every screen shows the same race.

use rand::Rng;
use rand::seq::{SliceRandom, index};
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 90;
pub const HORSES: usize = 6;
pub const YOURS: usize = 2;

pub const LABELS: [(i64, u32); 5] = [(100, 450), (150, 300), (250, 150), (500, 80), (2000, 20)];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bet {
    /// 1 to 6
    pub horse: u8,
    pub coins: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub bets: Vec<Bet>,
    /// Horses in the order they finish: the first one wins
    pub order: Vec<u8>,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let bets: Vec<Bet> = index::sample(rng, HORSES, YOURS)
        .into_iter()
        .map(|i| Bet { horse: i as u8 + 1, coins: *weighted(rng, &LABELS) })
        .collect();
    let mut order: Vec<u8> = (1..=HORSES as u8).collect();
    order.shuffle(rng);
    let prize = bets.iter().find(|b| b.horse == order[0]).map_or(Prize::Nothing, |b| Prize::Coins { amount: b.coins });
    Drawn { layout: serde_json::to_value(Layout { bets, order }).expect("serializes"), prize }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn the_winner_decides() {
        let mut rng = StdRng::seed_from_u64(31);
        for _ in 0..2000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            let mut order = l.order.clone();
            order.sort();
            assert_eq!(order, (1..=6).collect::<Vec<u8>>());
            assert_ne!(l.bets[0].horse, l.bets[1].horse);
            let expected = l
                .bets
                .iter()
                .find(|b| b.horse == l.order[0])
                .map_or(Prize::Nothing, |b| Prize::Coins { amount: b.coins });
            assert_eq!(d.prize, expected);
        }
    }
}
