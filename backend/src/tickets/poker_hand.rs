//! Poker Hand: five cards dealt from one deck, scratched all at once. The hand
//! pays by its rank, from a pair of jacks or better up to a royal flush.

use rand::Rng;
use rand::seq::index;
use serde::{Deserialize, Serialize};

use super::high_card::{Card, Suit};
use super::{Drawn, Prize};

pub const PRICE: i64 = 120;
pub const CARDS: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rank {
    Nothing,
    /// A pair of jacks, queens, kings or aces
    JacksOrBetter,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
    RoyalFlush,
}

/// What each hand pays
pub const PAYS: [(Rank, i64); 9] = [
    (Rank::JacksOrBetter, 200),
    (Rank::TwoPair, 600),
    (Rank::ThreeOfAKind, 1000),
    (Rank::Straight, 2000),
    (Rank::Flush, 3000),
    (Rank::FullHouse, 4000),
    (Rank::FourOfAKind, 15_000),
    (Rank::StraightFlush, 50_000),
    (Rank::RoyalFlush, 150_000),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub cards: Vec<Card>,
    pub rank: Rank,
}

const SUITS: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];

pub fn pays(rank: Rank) -> i64 {
    PAYS.iter().find(|(r, _)| *r == rank).map_or(0, |(_, c)| *c)
}

/// The best rank of five cards
pub fn rank(cards: &[Card]) -> Rank {
    let mut counts = [0u8; 15];
    for c in cards {
        counts[c.rank as usize] += 1;
    }
    let mut groups: Vec<(u8, u8)> =
        (2..=14u8).filter(|r| counts[*r as usize] > 0).map(|r| (counts[r as usize], r)).collect();
    groups.sort_by(|a, b| b.cmp(a));
    let flush = cards.iter().all(|c| c.suit == cards[0].suit);
    let mut ranks: Vec<u8> = cards.iter().map(|c| c.rank).collect();
    ranks.sort();
    let distinct = groups.len() == 5;
    // The ace plays low in A-2-3-4-5
    let wheel = ranks == [2, 3, 4, 5, 14];
    let straight = distinct && (ranks[4] - ranks[0] == 4 || wheel);
    match (straight, flush) {
        (true, true) if ranks[0] == 10 => return Rank::RoyalFlush,
        (true, true) => return Rank::StraightFlush,
        _ => {}
    }
    match (groups[0].0, groups.get(1).map(|g| g.0)) {
        (4, _) => Rank::FourOfAKind,
        (3, Some(2)) => Rank::FullHouse,
        _ if flush => Rank::Flush,
        _ if straight => Rank::Straight,
        (3, _) => Rank::ThreeOfAKind,
        (2, Some(2)) => Rank::TwoPair,
        (2, _) if groups[0].1 >= 11 => Rank::JacksOrBetter,
        _ => Rank::Nothing,
    }
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let cards: Vec<Card> = index::sample(rng, 52, CARDS)
        .into_iter()
        .map(|i| Card { rank: (i % 13) as u8 + 2, suit: SUITS[i / 13] })
        .collect();
    let rank = rank(&cards);
    let coins = pays(rank);
    let prize = if coins > 0 { Prize::Coins { amount: coins } } else { Prize::Nothing };
    Drawn { layout: serde_json::to_value(Layout { cards, rank }).expect("serializes"), prize }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hand(s: &str) -> Vec<Card> {
        s.split(' ')
            .map(|c| {
                let (r, s) = c.split_at(c.len() - 1);
                let rank = match r {
                    "J" => 11,
                    "Q" => 12,
                    "K" => 13,
                    "A" => 14,
                    n => n.parse().unwrap(),
                };
                let suit = match s {
                    "s" => Suit::Spades,
                    "h" => Suit::Hearts,
                    "d" => Suit::Diamonds,
                    _ => Suit::Clubs,
                };
                Card { rank, suit }
            })
            .collect()
    }

    #[test]
    fn hands_rank() {
        assert_eq!(rank(&hand("10s Js Qs Ks As")), Rank::RoyalFlush);
        assert_eq!(rank(&hand("5h 6h 7h 8h 9h")), Rank::StraightFlush);
        assert_eq!(rank(&hand("Ah 2c 3d 4s 5s")), Rank::Straight, "the ace plays low");
        assert_eq!(rank(&hand("9c 9d 9h 9s 2s")), Rank::FourOfAKind);
        assert_eq!(rank(&hand("9c 9d 9h 2c 2s")), Rank::FullHouse);
        assert_eq!(rank(&hand("2h 7h 9h Jh Kh")), Rank::Flush);
        assert_eq!(rank(&hand("4c 4d 4h Jc 2s")), Rank::ThreeOfAKind);
        assert_eq!(rank(&hand("4c 4d Jh Jc 2s")), Rank::TwoPair);
        assert_eq!(rank(&hand("Qc Qd 7h 5c 2s")), Rank::JacksOrBetter);
        assert_eq!(rank(&hand("10c 10d 7h 5c 2s")), Rank::Nothing, "tens are not enough");
        assert_eq!(rank(&hand("Kc Qd 7h 5c 2s")), Rank::Nothing);
    }

    #[test]
    fn the_layout_pays_its_rank() {
        let mut rng = rand::rng();
        for _ in 0..3000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            assert_eq!(l.cards.len(), CARDS);
            for (i, a) in l.cards.iter().enumerate() {
                assert!(l.cards[i + 1..].iter().all(|b| b != a), "one deck");
            }
            assert_eq!(l.rank, rank(&l.cards));
            assert_eq!(d.prize.coins(), pays(l.rank));
        }
    }
}
