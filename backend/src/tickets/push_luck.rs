//! Push Your Luck: twelve panels face down, three of them bombs. Flip them one
//! by one: each coin panel adds to the pot, a bomb loses it. Cash out whenever
//! you like (after the first flip).

use chrono::{DateTime, Utc};
use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use super::play::{Game, Outcome};
use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 60;
pub const PANELS: usize = 12;
pub const BOMBS: usize = 3;
pub const VALUES: [(i64, u32); 5] = [(10, 30), (20, 30), (40, 25), (80, 10), (250, 5)];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Panel {
    Coins { coins: i64 },
    Bomb,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub panels: Vec<Panel>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub flipped: Vec<usize>,
    pub cashed: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Move {
    Flip { panel: usize },
    Cash,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let mut panels: Vec<Panel> = (0..PANELS)
        .map(|i| if i < BOMBS { Panel::Bomb } else { Panel::Coins { coins: *weighted(rng, &VALUES) } })
        .collect();
    panels.shuffle(rng);
    Drawn { layout: serde_json::to_value(Layout { panels }).expect("serializes"), prize: Prize::Nothing }
}

pub fn pot(layout: &Layout, flipped: &[usize]) -> i64 {
    flipped
        .iter()
        .map(|&i| match layout.panels[i] {
            Panel::Coins { coins } => coins,
            Panel::Bomb => 0,
        })
        .sum()
}

fn boom(layout: &Layout, state: &State) -> bool {
    state.flipped.iter().any(|&i| layout.panels[i] == Panel::Bomb)
}

pub struct PushLuck;

impl Game for PushLuck {
    type Layout = Layout;
    type State = State;
    type Move = Move;

    fn view(layout: &Layout, state: &State, _now: DateTime<Utc>) -> serde_json::Value {
        let over = state.cashed || boom(layout, state);
        serde_json::json!({
            "flipped": state.flipped.iter().map(|&i| serde_json::json!({ "panel": i, "is": layout.panels[i] })).collect::<Vec<_>>(),
            "pot": pot(layout, &state.flipped),
            "boom": boom(layout, state),
            // Everything, once it is over: to see what was left
            "panels": if over { Some(&layout.panels) } else { None },
        })
    }

    fn play(layout: &Layout, state: &mut State, mv: Move, _now: DateTime<Utc>) -> Result<Outcome, &'static str> {
        match mv {
            Move::Flip { panel } => {
                if panel >= PANELS {
                    return Err("no_such_panel");
                }
                if state.flipped.contains(&panel) {
                    return Err("flipped_already");
                }
                state.flipped.push(panel);
                if layout.panels[panel] == Panel::Bomb {
                    return Ok(Outcome::Over(Prize::Nothing));
                }
                // Every coin panel flipped: nothing left to risk
                if state.flipped.len() == PANELS - BOMBS {
                    state.cashed = true;
                    return Ok(Outcome::Over(Prize::Coins { amount: pot(layout, &state.flipped) }));
                }
                Ok(Outcome::Going)
            }
            Move::Cash => {
                if state.flipped.is_empty() {
                    return Err("flip_first");
                }
                state.cashed = true;
                Ok(Outcome::Over(Prize::Coins { amount: pot(layout, &state.flipped) }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn layout() -> Layout {
        let mut panels = vec![Panel::Coins { coins: 10 }; PANELS];
        panels[0] = Panel::Bomb;
        panels[1] = Panel::Bomb;
        panels[2] = Panel::Bomb;
        Layout { panels }
    }

    #[test]
    fn flip_cash_and_boom() {
        let l = layout();
        let now = Utc::now();
        let mut s = State::default();
        assert_eq!(PushLuck::play(&l, &mut s, Move::Cash, now), Err("flip_first"));
        assert_eq!(PushLuck::play(&l, &mut s, Move::Flip { panel: 5 }, now), Ok(Outcome::Going));
        assert_eq!(PushLuck::play(&l, &mut s, Move::Flip { panel: 5 }, now), Err("flipped_already"));
        assert_eq!(PushLuck::play(&l, &mut s, Move::Flip { panel: 6 }, now), Ok(Outcome::Going));
        assert_eq!(PushLuck::play(&l, &mut s, Move::Cash, now), Ok(Outcome::Over(Prize::Coins { amount: 20 })));
        let mut s = State::default();
        PushLuck::play(&l, &mut s, Move::Flip { panel: 7 }, now).unwrap();
        assert_eq!(PushLuck::play(&l, &mut s, Move::Flip { panel: 1 }, now), Ok(Outcome::Over(Prize::Nothing)));
        // The hidden panels only show at the end
        let going = PushLuck::view(&l, &State { flipped: vec![7], cashed: false }, now);
        assert!(going["panels"].is_null());
        assert!(PushLuck::view(&l, &s, now)["panels"].is_array());
    }

    /// Whatever pot a player stops at, they get back at most 92 % of the price
    #[test]
    fn no_strategy_beats_the_house() {
        let mut rng = StdRng::seed_from_u64(5);
        let n = 30_000;
        let mut best: f64 = 0.0;
        for stop_at in [10, 30, 60, 100, 150, 200, 300, 500] {
            let mut total = 0.0;
            for _ in 0..n {
                let l: Layout = serde_json::from_value(draw(&mut rng).layout).unwrap();
                let mut order: Vec<usize> = (0..PANELS).collect();
                order.shuffle(&mut rng);
                let mut s = State::default();
                for p in order {
                    match PushLuck::play(&l, &mut s, Move::Flip { panel: p }, Utc::now()).unwrap() {
                        Outcome::Over(prize) => {
                            total += prize.coins() as f64;
                            break;
                        }
                        Outcome::Going if pot(&l, &s.flipped) >= stop_at => {
                            if let Outcome::Over(prize) = PushLuck::play(&l, &mut s, Move::Cash, Utc::now()).unwrap() {
                                total += prize.coins() as f64;
                            }
                            break;
                        }
                        Outcome::Going => {}
                    }
                }
            }
            best = best.max(total / n as f64 / PRICE as f64);
        }
        assert!((0.6..=0.92).contains(&best), "the best stopping point pays back {best:.2}");
    }
}
