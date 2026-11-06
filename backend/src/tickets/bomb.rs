//! Defuse the Bomb: six wires, twenty seconds. One wire defuses it and pays
//! the prize on its display, one sets it off, the others do nothing. The
//! clock starts when the ticket is first opened, and it is the server's.

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use super::play::{Game, Outcome};
use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 200;
pub const WIRES: usize = 6;
pub const FUSE: Duration = Duration::seconds(20);
pub const PRIZES: [(i64, u32); 5] = [(150, 40), (250, 30), (400, 20), (800, 8), (2500, 2)];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Wire {
    Defuse,
    Boom,
    Dud,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub wires: Vec<Wire>,
    pub coins: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub started: Option<DateTime<Utc>>,
    pub cut: Vec<usize>,
    /// Ran out of time
    pub late: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Move {
    pub cut: usize,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let mut wires = vec![Wire::Dud; WIRES];
    wires[0] = Wire::Defuse;
    wires[1] = Wire::Boom;
    wires.shuffle(rng);
    let layout = Layout { wires, coins: *weighted(rng, &PRIZES) };
    Drawn { layout: serde_json::to_value(layout).expect("serializes"), prize: Prize::Nothing }
}

fn over(l: &Layout, s: &State) -> bool {
    s.late || s.cut.iter().any(|&w| l.wires[w] != Wire::Dud)
}

pub struct Bomb;

impl Game for Bomb {
    type Layout = Layout;
    type State = State;
    type Move = Move;

    fn view(l: &Layout, s: &State, now: DateTime<Utc>) -> serde_json::Value {
        let left = s.started.map_or(FUSE, |t| (t + FUSE - now).max(Duration::zero()));
        serde_json::json!({
            "coins": l.coins,
            "cut": s.cut.iter().map(|&w| serde_json::json!({ "wire": w, "is": l.wires[w] })).collect::<Vec<_>>(),
            "ms_left": left.num_milliseconds(),
            "late": s.late,
            "wires": if over(l, s) { Some(&l.wires) } else { None },
        })
    }

    /// The fuse burns from the first look
    fn tick(l: &Layout, s: &mut State, now: DateTime<Utc>) -> Outcome {
        match s.started {
            None => {
                s.started = Some(now);
                Outcome::Going
            }
            Some(t) if now > t + FUSE && !over(l, s) => {
                s.late = true;
                Outcome::Over(Prize::Nothing)
            }
            _ => Outcome::Going,
        }
    }

    fn play(l: &Layout, s: &mut State, mv: Move, _now: DateTime<Utc>) -> Result<Outcome, &'static str> {
        if mv.cut >= WIRES {
            return Err("no_such_wire");
        }
        if s.cut.contains(&mv.cut) {
            return Err("cut_already");
        }
        s.cut.push(mv.cut);
        Ok(match l.wires[mv.cut] {
            Wire::Defuse => Outcome::Over(Prize::Coins { amount: l.coins }),
            Wire::Boom => Outcome::Over(Prize::Nothing),
            Wire::Dud => Outcome::Going,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tickets::Series;
    use crate::tickets::play::{Outcome, dispatch};

    fn bomb() -> serde_json::Value {
        serde_json::json!({ "wires": ["dud", "boom", "dud", "defuse", "dud", "dud"], "coins": 400 })
    }

    #[test]
    fn wires_and_the_fuse() {
        let t0: DateTime<Utc> = "2026-11-06T10:00:00Z".parse().unwrap();
        // First look lights the fuse
        let p = dispatch(Series::DefuseBomb, &bomb(), None, None, t0).unwrap().unwrap();
        assert_eq!(p.outcome, Outcome::Going);
        assert!(p.view["wires"].is_null(), "which wire is which stays hidden");
        let cut = |state: &serde_json::Value, w: usize, at| {
            dispatch(Series::DefuseBomb, &bomb(), Some(state), Some(&serde_json::json!({ "cut": w })), at).unwrap()
        };
        let p = cut(&p.state, 0, t0 + Duration::seconds(3)).unwrap();
        assert_eq!(p.outcome, Outcome::Going);
        assert_eq!(p.view["ms_left"], 17_000);
        let won = cut(&p.state, 3, t0 + Duration::seconds(5)).unwrap();
        assert_eq!(won.outcome, Outcome::Over(Prize::Coins { amount: 400 }));
        // Too late: the move comes after the fuse has burnt
        let late = cut(&p.state, 3, t0 + Duration::seconds(21)).unwrap();
        assert_eq!(late.outcome, Outcome::Over(Prize::Nothing));
        assert_eq!(late.view["late"], true);
        assert_eq!(cut(&p.state, 0, t0 + Duration::seconds(4)).unwrap_err(), "cut_already");
    }

    /// Cutting at random defuses it half the time: the payback stays below the price
    #[test]
    fn random_cuts_pay_back_less_than_the_price() {
        let mut rng = rand::rng();
        let n = 30_000;
        let mut total = 0;
        for _ in 0..n {
            let l: Layout = serde_json::from_value(draw(&mut rng).layout).unwrap();
            let first = l.wires.iter().position(|w| *w != Wire::Dud).unwrap();
            if l.wires[first] == Wire::Defuse {
                total += l.coins;
            }
        }
        let ratio = total as f64 / n as f64 / PRICE as f64;
        assert!((0.6..=0.92).contains(&ratio), "pays back {ratio:.2}");
    }
}
