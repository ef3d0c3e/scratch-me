//! Blackjack: three hands against the dealer, played out from one shuffled
//! deck (hands hit below 15, the dealer below 17). A hand closer to 21 than
//! the dealer's wins its prize; a blackjack pays double; a tie pays nothing.

use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use super::high_card::{Card, Suit};
use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 60;
pub const HANDS: usize = 3;
pub const LABELS: [(i64, u32); 5] = [(10, 40), (20, 30), (50, 20), (100, 8), (500, 1)];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Result_ {
    Blackjack,
    Win,
    Push,
    Lose,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hand {
    pub cards: Vec<Card>,
    pub coins: i64,
    pub result: Result_,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub dealer: Vec<Card>,
    pub hands: Vec<Hand>,
}

/// A hand's best total: aces count 11 unless that busts
pub fn total(cards: &[Card]) -> u8 {
    // Ranks run 2–14: picture cards count 10, the ace (14) 1 for now
    let mut sum: u8 = cards.iter().map(|c| if c.rank == 14 { 1 } else { c.rank.min(10) }).sum();
    let aces = cards.iter().filter(|c| c.rank == 14).count();
    for _ in 0..aces {
        if sum + 10 <= 21 {
            sum += 10;
        }
    }
    sum
}

fn deck(rng: &mut impl Rng) -> Vec<Card> {
    let suits = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];
    let mut d: Vec<Card> = suits.iter().flat_map(|&suit| (2..=14).map(move |rank| Card { rank, suit })).collect();
    d.shuffle(rng);
    d
}

pub fn judge(hand: &[Card], dealer: &[Card]) -> Result_ {
    let (p, d) = (total(hand), total(dealer));
    let natural = |c: &[Card]| c.len() == 2 && total(c) == 21;
    if p > 21 {
        Result_::Lose
    } else if natural(hand) && !natural(dealer) {
        Result_::Blackjack
    } else if d > 21 || p > d {
        Result_::Win
    } else if p == d {
        Result_::Push
    } else {
        Result_::Lose
    }
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let mut d = deck(rng);
    let mut take = || d.pop().expect("cards left");
    let mut hands: Vec<Vec<Card>> = (0..HANDS).map(|_| vec![take(), take()]).collect();
    for h in hands.iter_mut() {
        while total(h) < 15 {
            h.push(take());
        }
    }
    let mut dealer = vec![take(), take()];
    while total(&dealer) < 17 {
        dealer.push(take());
    }
    let hands: Vec<Hand> = hands
        .into_iter()
        .map(|cards| {
            let result = judge(&cards, &dealer);
            Hand { cards, coins: *weighted(rng, &LABELS), result }
        })
        .collect();
    let total: i64 = hands
        .iter()
        .map(|h| match h.result {
            Result_::Blackjack => h.coins * 2,
            Result_::Win => h.coins,
            _ => 0,
        })
        .sum();
    let prize = if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing };
    Drawn { layout: serde_json::to_value(Layout { dealer, hands }).expect("serializes"), prize }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(rank: u8) -> Card {
        Card { rank, suit: Suit::Spades }
    }

    #[test]
    fn totals_and_judging() {
        assert_eq!(total(&[c(14), c(13)]), 21);
        assert_eq!(total(&[c(14), c(14), c(9)]), 21);
        assert_eq!(total(&[c(12), c(11), c(5)]), 25);
        assert_eq!(judge(&[c(14), c(12)], &[c(10), c(9)]), Result_::Blackjack);
        assert_eq!(judge(&[c(10), c(9)], &[c(10), c(9)]), Result_::Push);
        assert_eq!(judge(&[c(10), c(5), c(9)], &[c(10), c(9)]), Result_::Lose);
        assert_eq!(judge(&[c(10), c(5)], &[c(10), c(6), c(10)]), Result_::Win);
    }

    #[test]
    fn hands_show_their_result() {
        let mut rng = rand::rng();
        for _ in 0..2000 {
            let l: Layout = serde_json::from_value(draw(&mut rng).layout).unwrap();
            assert!(total(&l.dealer) >= 17);
            for h in &l.hands {
                assert_eq!(h.result, judge(&h.cards, &l.dealer));
            }
        }
    }
}
