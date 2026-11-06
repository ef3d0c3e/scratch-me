//! Safe Cracker: a three-digit code, five tries. After each try every digit
//! says whether the code's is higher, lower, or right. The sooner the safe
//! opens, the more it pays.

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::play::{Game, Outcome};
use super::{Drawn, Prize};

pub const PRICE: i64 = 300;
pub const DIGITS: usize = 3;
/// What opening on the 1st, 2nd… try pays
pub const PAYS: [i64; 5] = [3000, 1000, 400, 150, 60];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub code: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hint {
    Right,
    Higher,
    Lower,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub tries: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Move {
    pub guess: Vec<u8>,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let code = (0..DIGITS).map(|_| rng.random_range(0..10)).collect();
    Drawn { layout: serde_json::to_value(Layout { code }).expect("serializes"), prize: Prize::Nothing }
}

pub fn hints(code: &[u8], guess: &[u8]) -> Vec<Hint> {
    code.iter()
        .zip(guess)
        .map(|(c, g)| match g.cmp(c) {
            std::cmp::Ordering::Equal => Hint::Right,
            std::cmp::Ordering::Less => Hint::Higher,
            std::cmp::Ordering::Greater => Hint::Lower,
        })
        .collect()
}

pub struct SafeCracker;

impl Game for SafeCracker {
    type Layout = Layout;
    type State = State;
    type Move = Move;

    fn view(layout: &Layout, state: &State, _now: DateTime<Utc>) -> serde_json::Value {
        let opened = state.tries.last().is_some_and(|t| *t == layout.code);
        let over = opened || state.tries.len() >= PAYS.len();
        serde_json::json!({
            "tries": state.tries.iter().map(|t| serde_json::json!({ "guess": t, "hints": hints(&layout.code, t) })).collect::<Vec<_>>(),
            "left": PAYS.len() - state.tries.len(),
            "pays": PAYS,
            "opened": opened,
            "code": if over { Some(&layout.code) } else { None },
        })
    }

    fn play(layout: &Layout, state: &mut State, mv: Move, _now: DateTime<Utc>) -> Result<Outcome, &'static str> {
        if mv.guess.len() != DIGITS || mv.guess.iter().any(|d| *d > 9) {
            return Err("bad_code");
        }
        state.tries.push(mv.guess.clone());
        if mv.guess == layout.code {
            return Ok(Outcome::Over(Prize::Coins { amount: PAYS[state.tries.len() - 1] }));
        }
        if state.tries.len() >= PAYS.len() {
            return Ok(Outcome::Over(Prize::Nothing));
        }
        Ok(Outcome::Going)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn hints_and_tries() {
        let l = Layout { code: vec![3, 7, 0] };
        assert_eq!(hints(&l.code, &[5, 7, 1]), vec![Hint::Lower, Hint::Right, Hint::Lower]);
        let mut s = State::default();
        let now = Utc::now();
        assert_eq!(SafeCracker::play(&l, &mut s, Move { guess: vec![1, 2] }, now), Err("bad_code"));
        for _ in 0..4 {
            assert_eq!(SafeCracker::play(&l, &mut s, Move { guess: vec![0, 0, 0] }, now), Ok(Outcome::Going));
        }
        assert!(SafeCracker::view(&l, &s, now)["code"].is_null(), "the code stays hidden");
        assert_eq!(
            SafeCracker::play(&l, &mut s, Move { guess: vec![3, 7, 0] }, now),
            Ok(Outcome::Over(Prize::Coins { amount: 60 }))
        );
    }

    /// The best a player can do, halving each digit's range, pays back at most 92 %
    #[test]
    fn perfect_play_still_pays_less_than_the_price() {
        let mut rng = StdRng::seed_from_u64(9);
        let n = 20_000;
        let mut total = 0;
        for _ in 0..n {
            let l: Layout = serde_json::from_value(draw(&mut rng).layout).unwrap();
            let mut lo = [0u8; DIGITS];
            let mut hi = [9u8; DIGITS];
            let mut s = State::default();
            loop {
                let guess: Vec<u8> = (0..DIGITS).map(|i| (lo[i] + hi[i]) / 2).collect();
                match SafeCracker::play(&l, &mut s, Move { guess: guess.clone() }, Utc::now()).unwrap() {
                    Outcome::Over(p) => {
                        total += p.coins();
                        break;
                    }
                    Outcome::Going => {
                        for (i, h) in hints(&l.code, &guess).into_iter().enumerate() {
                            match h {
                                Hint::Higher => lo[i] = guess[i] + 1,
                                Hint::Lower => hi[i] = guess[i] - 1,
                                Hint::Right => (lo[i], hi[i]) = (guess[i], guess[i]),
                            }
                        }
                    }
                }
            }
        }
        let ratio = total as f64 / n as f64 / PRICE as f64;
        assert!((0.6..=0.92).contains(&ratio), "perfect play pays back {ratio:.2}");
    }
}
