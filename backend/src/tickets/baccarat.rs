//! Baccarat: Player, Banker and Tie each hide a bet. The player scratches one
//! (their call and the bet they placed), then the two hands are dealt by the
//! real rules (punto banco, third card included). The cards and the bets are
//! drawn when the ticket is issued; the bets are as much bigger as their side
//! is rarer, so every call is worth the same.

use chrono::{DateTime, Utc};
use rand::Rng;
use rand::seq::index;
use serde::{Deserialize, Serialize};

use super::high_card::{Card, Suit};
use super::play::{Game, Outcome};
use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 90;

/// Player and Banker bets (each side wins about 45 % of hands)
pub const SIDE_BETS: [(i64, u32); 4] = [(140, 50), (170, 30), (220, 15), (400, 5)];
/// Tie bets: a tie is about 1 hand in 10
pub const TIE_BETS: [(i64, u32); 3] = [(600, 60), (800, 30), (1200, 10)];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Player,
    Banker,
    Tie,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// The bets under Player, Banker and Tie
    pub bets: [i64; 3],
    pub player: Vec<Card>,
    pub banker: Vec<Card>,
    pub winner: Side,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub call: Option<Side>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Move {
    pub side: Side,
}

const SUITS: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];

/// A card's baccarat value: aces 1, tens and faces 0
fn value(c: &Card) -> u8 {
    match c.rank {
        14 => 1,
        10..=13 => 0,
        r => r,
    }
}

pub fn total(hand: &[Card]) -> u8 {
    hand.iter().map(value).sum::<u8>() % 10
}

/// Deals a coup from these cards, in order: the hands and who won
pub fn deal(cards: &[Card]) -> (Vec<Card>, Vec<Card>, Side) {
    let mut next = cards.iter().copied();
    let mut take = || next.next().expect("six cards are enough");
    let mut player = vec![take(), take()];
    let mut banker = vec![take(), take()];
    let (p, b) = (total(&player), total(&banker));
    if p < 8 && b < 8 {
        // Player draws on 0–5
        let third = (p <= 5).then(|| {
            let c = take();
            player.push(c);
            value(&c)
        });
        let b = total(&banker);
        let banker_draws = match third {
            None => b <= 5,
            Some(t) => match b {
                0..=2 => true,
                3 => t != 8,
                4 => (2..=7).contains(&t),
                5 => (4..=7).contains(&t),
                6 => t == 6 || t == 7,
                _ => false,
            },
        };
        if banker_draws {
            banker.push(take());
        }
    }
    let (p, b) = (total(&player), total(&banker));
    let winner = if p > b {
        Side::Player
    } else if b > p {
        Side::Banker
    } else {
        Side::Tie
    };
    (player, banker, winner)
}

fn at(side: Side) -> usize {
    match side {
        Side::Player => 0,
        Side::Banker => 1,
        Side::Tie => 2,
    }
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let cards: Vec<Card> =
        index::sample(rng, 52, 6).into_iter().map(|i| Card { rank: (i % 13) as u8 + 2, suit: SUITS[i / 13] }).collect();
    let (player, banker, winner) = deal(&cards);
    let bets = [*weighted(rng, &SIDE_BETS), *weighted(rng, &SIDE_BETS), *weighted(rng, &TIE_BETS)];
    let layout = Layout { bets, player, banker, winner };
    Drawn { layout: serde_json::to_value(layout).expect("serializes"), prize: Prize::Nothing }
}

pub fn prize_of(l: &Layout, call: Side) -> Prize {
    if l.winner == call { Prize::Coins { amount: l.bets[at(call)] } } else { Prize::Nothing }
}

pub struct Baccarat;

impl Game for Baccarat {
    type Layout = Layout;
    type State = State;
    type Move = Move;

    fn view(l: &Layout, s: &State, _now: DateTime<Utc>) -> serde_json::Value {
        match s.call {
            None => serde_json::json!({}),
            Some(call) => serde_json::json!({
                "call": call, "bets": l.bets, "player": l.player, "banker": l.banker, "winner": l.winner,
                "totals": [total(&l.player), total(&l.banker)],
            }),
        }
    }

    fn play(l: &Layout, s: &mut State, mv: Move, _now: DateTime<Utc>) -> Result<Outcome, &'static str> {
        if s.call.is_some() {
            return Err("called_already");
        }
        s.call = Some(mv.side);
        Ok(Outcome::Over(prize_of(l, mv.side)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn c(rank: u8) -> Card {
        Card { rank, suit: Suit::Spades }
    }

    #[test]
    fn the_rules_of_the_coup() {
        // A natural stands: player 8 (3 + 5), banker 7
        let (p, b, w) = deal(&[c(3), c(5), c(10), c(7), c(9), c(9)]);
        assert_eq!((p.len(), b.len(), w), (2, 2, Side::Player));
        // Player 5 draws a 2 (7); banker 3 draws (third card is not an 8)
        let (p, b, _) = deal(&[c(2), c(3), c(13), c(3), c(2), c(4)]);
        assert_eq!((total(&p), b.len()), (7, 3));
        // Banker 6 draws only on a player's third 6 or 7
        let (_, b, _) = deal(&[c(14), c(14), c(6), c(10), c(5), c(9)]);
        assert_eq!(b.len(), 2, "player's third card is a 5");
        assert_eq!(total(&[c(14), c(12)]), 1, "an ace counts 1, a face 0");
    }

    /// Every call pays back 70–90 % of the price
    #[test]
    fn every_call_pays_most_of_the_price() {
        let mut rng = StdRng::seed_from_u64(9);
        for call in [Side::Player, Side::Banker, Side::Tie] {
            let n = 300_000;
            let total: i64 = (0..n)
                .map(|_| {
                    let l: Layout = serde_json::from_value(draw(&mut rng).layout).unwrap();
                    prize_of(&l, call).coins()
                })
                .sum();
            let ratio = total as f64 / n as f64 / PRICE as f64;
            assert!((0.70..=0.90).contains(&ratio), "{call:?} pays back {ratio:.3}");
        }
    }
}
