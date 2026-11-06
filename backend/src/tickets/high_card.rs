//! High Card: the dealer's card and four of yours, each with a prize under
//! it. Every card of yours higher than the dealer's pays its prize; a tie loses.

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 50;
pub const HANDS: usize = 4;
/// Ranks run from 2 to 14 (the ace)
pub const LOW: u8 = 2;
pub const HIGH: u8 = 14;

pub const LABELS: [(i64, u32); 6] = [(5, 400), (10, 300), (20, 180), (50, 80), (200, 30), (500, 10)];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

const SUITS: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub rank: u8,
    pub suit: Suit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hand {
    pub card: Card,
    pub coins: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub dealer: Card,
    pub hands: Vec<Hand>,
}

fn card(rng: &mut impl Rng) -> Card {
    Card { rank: rng.random_range(LOW..=HIGH), suit: SUITS[rng.random_range(0..4)] }
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let dealer = card(rng);
    let hands: Vec<Hand> = (0..HANDS).map(|_| Hand { card: card(rng), coins: *weighted(rng, &LABELS) }).collect();
    let total: i64 = hands.iter().filter(|h| h.card.rank > dealer.rank).map(|h| h.coins).sum();
    let prize = if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing };
    Drawn { layout: serde_json::to_value(Layout { dealer, hands }).expect("serializes"), prize }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn higher_cards_pay_their_prize() {
        let mut rng = StdRng::seed_from_u64(8);
        for _ in 0..3000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            assert_eq!(l.hands.len(), HANDS);
            assert!((LOW..=HIGH).contains(&l.dealer.rank));
            let total: i64 = l.hands.iter().filter(|h| h.card.rank > l.dealer.rank).map(|h| h.coins).sum();
            assert_eq!(d.prize, if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing });
        }
    }
}
