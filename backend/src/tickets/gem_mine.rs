//! Gem Mine: a 5×5 mine, three of its cells hide a mine, the rest gems. Each
//! cell has its own little foil: every gem found raises what cashing out pays;
//! a mine loses it all. The mine is laid out when the ticket is issued.
//!
//! What k gems pay is set so that stopping after any k is worth the same
//! (the chance of k gems in a row, times the payout, is PAYBACK × the price):
//! where the mines are is random and nothing shows it, so no way of playing
//! beats another.

use chrono::{DateTime, Utc};
use rand::Rng;
use rand::seq::index;
use serde::{Deserialize, Serialize};

use super::play::{Game, Outcome};
use super::{Drawn, Prize};

pub const PRICE: i64 = 200;
pub const CELLS: usize = 25;
pub const MINES: usize = 3;
const PAYBACK: f64 = 0.86;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub mines: Vec<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub dug: Vec<usize>,
    pub boom: bool,
    pub cashed: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Move {
    Dig { cell: usize },
    Cash,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let mines = index::sample(rng, CELLS, MINES).into_vec();
    Drawn { layout: serde_json::to_value(Layout { mines }).expect("serializes"), prize: Prize::Nothing }
}

/// The chance of `k` gems in a row
fn safe(k: usize) -> f64 {
    (0..k).map(|i| (CELLS - MINES - i) as f64 / (CELLS - i) as f64).product()
}

/// What cashing out pays after `k` gems, rounded down
pub fn pays(k: usize) -> i64 {
    if k == 0 { 0 } else { (PRICE as f64 * PAYBACK / safe(k)).floor() as i64 }
}

fn gems(s: &State) -> usize {
    s.dug.len() - usize::from(s.boom)
}

pub struct GemMine;

impl Game for GemMine {
    type Layout = Layout;
    type State = State;
    type Move = Move;

    fn view(l: &Layout, s: &State, _now: DateTime<Utc>) -> serde_json::Value {
        let over = s.boom || s.cashed || gems(s) == CELLS - MINES;
        let k = gems(s);
        serde_json::json!({
            "dug": s.dug.iter().map(|c| serde_json::json!({ "cell": c, "mine": l.mines.contains(c) })).collect::<Vec<_>>(),
            "gems": k,
            "now": pays(k),
            "next": if k < CELLS - MINES { pays(k + 1) } else { 0 },
            "boom": s.boom,
            "cashed": s.cashed,
            "mines": if over { Some(&l.mines) } else { None },
        })
    }

    fn play(l: &Layout, s: &mut State, mv: Move, _now: DateTime<Utc>) -> Result<Outcome, &'static str> {
        match mv {
            Move::Cash => {
                if gems(s) == 0 {
                    return Err("nothing_to_cash");
                }
                s.cashed = true;
                Ok(Outcome::Over(Prize::Coins { amount: pays(gems(s)) }))
            }
            Move::Dig { cell } => {
                if cell >= CELLS {
                    return Err("no_such_cell");
                }
                if s.dug.contains(&cell) {
                    return Err("dug_already");
                }
                s.dug.push(cell);
                if l.mines.contains(&cell) {
                    s.boom = true;
                    return Ok(Outcome::Over(Prize::Nothing));
                }
                Ok(if gems(s) == CELLS - MINES {
                    Outcome::Over(Prize::Coins { amount: pays(gems(s)) })
                } else {
                    Outcome::Going
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn gems_raise_the_payout_and_a_mine_ends_it() {
        assert_eq!(pays(0), 0);
        assert_eq!(pays(1), (PRICE as f64 * PAYBACK / (22.0 / 25.0)) as i64);
        assert!((1..22).all(|k| pays(k + 1) > pays(k)));
        let l = Layout { mines: vec![0, 1, 2] };
        let mut s = State::default();
        let now = Utc::now();
        assert_eq!(GemMine::play(&l, &mut s, Move::Cash, now), Err("nothing_to_cash"));
        assert_eq!(GemMine::play(&l, &mut s, Move::Dig { cell: 10 }, now), Ok(Outcome::Going));
        assert!(GemMine::view(&l, &s, now)["mines"].is_null(), "the mines stay hidden");
        assert_eq!(GemMine::play(&l, &mut s, Move::Dig { cell: 10 }, now), Err("dug_already"));
        assert_eq!(GemMine::play(&l, &mut s, Move::Dig { cell: 1 }, now), Ok(Outcome::Over(Prize::Nothing)));
    }

    /// Stopping after any number of gems pays back the same: 70–90 %
    #[test]
    fn any_way_of_playing_pays_most_of_the_price() {
        let mut rng = StdRng::seed_from_u64(3);
        let now = Utc::now();
        for stop in [1, 3, 6, 10] {
            let n = 200_000;
            let total: i64 = (0..n)
                .map(|_| {
                    let l: Layout = serde_json::from_value(draw(&mut rng).layout).unwrap();
                    let mut s = State::default();
                    for cell in 0..stop {
                        if let Outcome::Over(_) = GemMine::play(&l, &mut s, Move::Dig { cell }, now).unwrap() {
                            return 0;
                        }
                    }
                    GemMine::play(&l, &mut s, Move::Cash, now)
                        .map(|o| match o {
                            Outcome::Over(p) => p.coins(),
                            Outcome::Going => 0,
                        })
                        .unwrap()
                })
                .sum();
            let ratio = total as f64 / n as f64 / PRICE as f64;
            assert!((0.70..=0.90).contains(&ratio), "stopping after {stop} pays back {ratio:.3}");
        }
    }
}
