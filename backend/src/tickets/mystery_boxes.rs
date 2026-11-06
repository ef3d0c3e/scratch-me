//! Mystery Boxes: three boxes, one holds the prize. Pick one; the host opens
//! an empty one of the other two; keep yours or switch. (Switching wins two
//! times out of three.)

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::play::{Game, Outcome};
use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 260;
pub const BOXES: usize = 3;
pub const PRIZES: [(i64, u32); 4] = [(150, 50), (300, 30), (600, 15), (1500, 5)];
/// The box that is neither the prize nor opened by the host holds this
pub const CONSOLATION: i64 = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub prize_box: usize,
    pub coins: i64,
    /// When the player picked the prize, which other box the host opens (0 or 1
    /// among the other two): drawn now so the host gives nothing away
    pub host: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub picked: Option<usize>,
    pub opened: Option<usize>,
    pub last: Option<usize>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Move {
    Pick { pick: usize },
    Keep,
    Switch,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let layout =
        Layout { prize_box: rng.random_range(0..BOXES), coins: *weighted(rng, &PRIZES), host: rng.random_range(0..2) };
    Drawn { layout: serde_json::to_value(layout).expect("serializes"), prize: Prize::Nothing }
}

fn host_opens(l: &Layout, picked: usize) -> usize {
    let others: Vec<usize> = (0..BOXES).filter(|b| *b != picked).collect();
    if picked == l.prize_box {
        others[l.host]
    } else {
        *others.iter().find(|b| **b != l.prize_box).expect("an empty box")
    }
}

pub fn prize_of(l: &Layout, last: usize) -> Prize {
    Prize::Coins { amount: if last == l.prize_box { l.coins } else { CONSOLATION } }
}

pub struct MysteryBoxes;

impl Game for MysteryBoxes {
    type Layout = Layout;
    type State = State;
    type Move = Move;

    fn view(l: &Layout, s: &State, _now: DateTime<Utc>) -> serde_json::Value {
        serde_json::json!({
            "picked": s.picked,
            "opened": s.opened,
            "last": s.last,
            // The prize's place and size only once it is over
            "prize_box": s.last.map(|_| l.prize_box),
            "coins": s.last.map(|_| l.coins),
            "consolation": CONSOLATION,
        })
    }

    fn play(l: &Layout, s: &mut State, mv: Move, _now: DateTime<Utc>) -> Result<Outcome, &'static str> {
        match (mv, s.picked) {
            (Move::Pick { pick }, None) => {
                if pick >= BOXES {
                    return Err("no_such_box");
                }
                s.picked = Some(pick);
                s.opened = Some(host_opens(l, pick));
                Ok(Outcome::Going)
            }
            (Move::Keep | Move::Switch, Some(picked)) => {
                let last = if let Move::Switch = mv {
                    (0..BOXES).find(|b| *b != picked && Some(*b) != s.opened).expect("the third box")
                } else {
                    picked
                };
                s.last = Some(last);
                Ok(Outcome::Over(prize_of(l, last)))
            }
            (Move::Pick { .. }, Some(_)) => Err("picked_already"),
            (_, None) => Err("pick_first"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn the_host_never_opens_the_prize_or_your_box() {
        let mut rng = StdRng::seed_from_u64(2);
        for _ in 0..2000 {
            let l: Layout = serde_json::from_value(draw(&mut rng).layout).unwrap();
            for pick in 0..BOXES {
                let o = host_opens(&l, pick);
                assert!(o != pick && o != l.prize_box);
            }
        }
    }

    #[test]
    fn switching_wins_two_times_in_three_and_still_pays_less_than_the_price() {
        let mut rng = StdRng::seed_from_u64(3);
        let n = 30_000;
        let (mut wins, mut total) = (0, 0);
        for _ in 0..n {
            let l: Layout = serde_json::from_value(draw(&mut rng).layout).unwrap();
            let mut s = State::default();
            MysteryBoxes::play(&l, &mut s, Move::Pick { pick: rng.random_range(0..3) }, Utc::now()).unwrap();
            assert!(MysteryBoxes::view(&l, &s, Utc::now())["prize_box"].is_null());
            if let Outcome::Over(p) = MysteryBoxes::play(&l, &mut s, Move::Switch, Utc::now()).unwrap() {
                total += p.coins();
                if p.coins() > CONSOLATION {
                    wins += 1;
                }
            }
        }
        let rate = wins as f64 / n as f64;
        assert!((rate - 2.0 / 3.0).abs() < 0.02, "switching wins {rate:.2}");
        let ratio = total as f64 / n as f64 / PRICE as f64;
        assert!((0.6..=0.92).contains(&ratio), "always switching pays back {ratio:.2}");
    }
}
