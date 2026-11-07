//! Coin Flip: two cells, Heads and Tails, each hiding a bet. The player
//! scratches one (that is their call and the bet they placed), then the coin
//! is flipped. Both bets and the flip are drawn when the ticket is issued;
//! the bets are hidden, so either call is worth the same.

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::play::{Game, Outcome};
use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 40;

/// What a bet pays when the coin agrees, and how often it is that bet
pub const BETS: [(i64, u32); 5] = [(50, 50), (60, 25), (80, 15), (120, 7), (250, 3)];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Heads,
    Tails,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// The bet under Heads, then under Tails
    pub bets: [i64; 2],
    pub flip: Side,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub call: Option<Side>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Move {
    pub side: Side,
}

fn at(side: Side) -> usize {
    match side {
        Side::Heads => 0,
        Side::Tails => 1,
    }
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let layout = Layout {
        bets: [*weighted(rng, &BETS), *weighted(rng, &BETS)],
        flip: if rng.random_bool(0.5) { Side::Heads } else { Side::Tails },
    };
    // The prize comes from the call
    Drawn { layout: serde_json::to_value(layout).expect("serializes"), prize: Prize::Nothing }
}

pub fn prize_of(l: &Layout, call: Side) -> Prize {
    if l.flip == call { Prize::Coins { amount: l.bets[at(call)] } } else { Prize::Nothing }
}

pub struct CoinFlip;

impl Game for CoinFlip {
    type Layout = Layout;
    type State = State;
    type Move = Move;

    fn view(l: &Layout, s: &State, _now: DateTime<Utc>) -> serde_json::Value {
        match s.call {
            None => serde_json::json!({}),
            // Called: everything shows, the other bet too
            Some(call) => serde_json::json!({ "call": call, "bets": l.bets, "flip": l.flip }),
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

    #[test]
    fn the_call_wins_its_bet_when_the_coin_agrees() {
        let l = Layout { bets: [50, 250], flip: Side::Tails };
        assert_eq!(prize_of(&l, Side::Tails), Prize::Coins { amount: 250 });
        assert_eq!(prize_of(&l, Side::Heads), Prize::Nothing);
        let mut s = State::default();
        let now = Utc::now();
        assert_eq!(CoinFlip::view(&l, &s, now), serde_json::json!({}), "nothing shows before the call");
        assert_eq!(
            CoinFlip::play(&l, &mut s, Move { side: Side::Tails }, now),
            Ok(Outcome::Over(prize_of(&l, Side::Tails)))
        );
        assert_eq!(CoinFlip::play(&l, &mut s, Move { side: Side::Heads }, now), Err("called_already"));
    }

    /// Either call pays back 70–90 % of the price
    #[test]
    fn either_call_pays_most_of_the_price() {
        let mut rng = StdRng::seed_from_u64(4);
        for call in [Side::Heads, Side::Tails] {
            let n = 100_000;
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
