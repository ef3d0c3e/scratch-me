//! Higher or Lower: a card shows; the player guesses whether the next is
//! higher or lower. Every right guess climbs the ladder; a wrong one (or the
//! same rank) loses it all. Cash out any time after a right guess. The cards
//! are drawn when the ticket is issued: the server knows them, the player
//! only sees each once guessed.

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::high_card::{Card, Suit};
use super::play::{Game, Outcome};
use super::{Drawn, Prize};

pub const PRICE: i64 = 50;
/// What cashing out pays after this many right guesses (index 0: none yet)
pub const LADDER: [i64; 8] = [0, 55, 78, 110, 155, 220, 310, 440];
/// Guesses there can be: the ladder's top pays itself
pub const GUESSES: usize = LADDER.len() - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Guess {
    Higher,
    Lower,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// The first card, then one per guess
    pub cards: Vec<Card>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// Right guesses so far
    pub right: usize,
    pub lost: bool,
    pub cashed: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Move {
    Guess { guess: Guess },
    Cash,
}

const SUITS: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let cards =
        (0..=GUESSES).map(|_| Card { rank: rng.random_range(2..=14), suit: SUITS[rng.random_range(0..4)] }).collect();
    Drawn { layout: serde_json::to_value(Layout { cards }).expect("serializes"), prize: Prize::Nothing }
}

/// Whether a guess is right: strictly higher or lower, the same rank loses
pub fn right(from: &Card, to: &Card, guess: Guess) -> bool {
    match guess {
        Guess::Higher => to.rank > from.rank,
        Guess::Lower => to.rank < from.rank,
    }
}

fn pays(right: usize) -> Prize {
    let coins = LADDER[right.min(GUESSES)];
    if coins > 0 { Prize::Coins { amount: coins } } else { Prize::Nothing }
}

pub struct HigherLower;

impl Game for HigherLower {
    type Layout = Layout;
    type State = State;
    type Move = Move;

    fn view(l: &Layout, s: &State, _now: DateTime<Utc>) -> serde_json::Value {
        let over = s.lost || s.cashed || s.right >= GUESSES;
        // The cards seen: the first, one per right guess, and the one that lost
        let seen = 1 + s.right + usize::from(s.lost);
        serde_json::json!({
            "cards": if over { &l.cards[..] } else { &l.cards[..seen] },
            "seen": seen,
            "right": s.right,
            "ladder": LADDER,
            "lost": s.lost,
            "cashed": s.cashed,
        })
    }

    fn play(l: &Layout, s: &mut State, mv: Move, _now: DateTime<Utc>) -> Result<Outcome, &'static str> {
        match mv {
            Move::Cash => {
                if s.right == 0 {
                    return Err("nothing_to_cash");
                }
                s.cashed = true;
                Ok(Outcome::Over(pays(s.right)))
            }
            Move::Guess { guess } => {
                if right(&l.cards[s.right], &l.cards[s.right + 1], guess) {
                    s.right += 1;
                    Ok(if s.right >= GUESSES { Outcome::Over(pays(s.right)) } else { Outcome::Going })
                } else {
                    s.lost = true;
                    Ok(Outcome::Over(Prize::Nothing))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    /// The likelier way from a rank, and its chance (ties lose)
    fn best(rank: u8) -> (Guess, f64) {
        let higher = (14 - rank) as f64 / 13.0;
        let lower = (rank - 2) as f64 / 13.0;
        if higher >= lower { (Guess::Higher, higher) } else { (Guess::Lower, lower) }
    }

    /// Plays a ticket: guess the likelier way; `stop` says when to cash out
    fn play_out(l: &Layout, stop: &dyn Fn(usize, f64) -> bool) -> i64 {
        let mut s = State::default();
        let now = Utc::now();
        loop {
            let (guess, p) = best(l.cards[s.right].rank);
            if s.right > 0 && stop(s.right, p) {
                return HigherLower::play(l, &mut s, Move::Cash, now).unwrap().prize().coins();
            }
            match HigherLower::play(l, &mut s, Move::Guess { guess }, now).unwrap() {
                Outcome::Going => {}
                Outcome::Over(p) => return p.coins(),
            }
        }
    }

    trait PrizeOf {
        fn prize(self) -> Prize;
    }
    impl PrizeOf for Outcome {
        fn prize(self) -> Prize {
            match self {
                Outcome::Over(p) => p,
                Outcome::Going => Prize::Nothing,
            }
        }
    }

    fn payback(stop: &dyn Fn(usize, f64) -> bool) -> f64 {
        let mut rng = StdRng::seed_from_u64(4);
        let n = 200_000;
        let total: i64 = (0..n).map(|_| play_out(&serde_json::from_value(draw(&mut rng).layout).unwrap(), stop)).sum();
        total as f64 / n as f64 / PRICE as f64
    }

    #[test]
    fn guesses_climb_and_a_miss_or_a_tie_loses() {
        let c = |rank| Card { rank, suit: Suit::Hearts };
        let l = Layout { cards: vec![c(7), c(9), c(9), c(2), c(3), c(4), c(5), c(6)] };
        let mut s = State::default();
        let now = Utc::now();
        assert_eq!(HigherLower::play(&l, &mut s, Move::Cash, now), Err("nothing_to_cash"));
        assert_eq!(HigherLower::play(&l, &mut s, Move::Guess { guess: Guess::Higher }, now), Ok(Outcome::Going));
        assert_eq!(HigherLower::view(&l, &s, now)["cards"].as_array().unwrap().len(), 2, "the next card stays hidden");
        // 9 then 9: the same rank loses
        assert_eq!(
            HigherLower::play(&l, &mut s, Move::Guess { guess: Guess::Higher }, now),
            Ok(Outcome::Over(Prize::Nothing))
        );
    }

    /// The best play (the likelier way, cashing out when going on is worth
    /// less) pays back at most 92 %; a cautious player still gets most
    #[test]
    fn the_ladder_pays_most_of_the_price_and_never_more() {
        // Go on while the expected next rung beats what is in hand
        let smart = |right: usize, p: f64| p * LADDER[(right + 1).min(GUESSES)] as f64 <= LADDER[right] as f64;
        let cautious = |right: usize, _p: f64| right >= 2;
        let (smart, cautious) = (payback(&smart), payback(&cautious));
        eprintln!("smart {smart:.3}, cautious {cautious:.3}");
        assert!((0.70..=0.92).contains(&smart), "best play pays back {smart:.3}");
        assert!((0.55..=0.92).contains(&cautious), "cashing out at 2 pays back {cautious:.3}");
    }
}
