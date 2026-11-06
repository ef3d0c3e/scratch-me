//! Rocket: launch it and the multiplier climbs, doubling every four seconds.
//! It crashes at a point drawn with the ticket; cash out before, and you get
//! the stake times the multiplier the server measured at that moment.

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::play::{Game, Outcome};
use super::{Drawn, Prize};

pub const PRICE: i64 = 100;
pub const STAKE: i64 = 100;
/// Seconds for the multiplier to double
pub const DOUBLING: f64 = 4.0;
/// Chance the rocket is still flying at multiplier m: HOUSE / m (so any way of
/// playing pays back HOUSE of the stake)
pub const HOUSE: f64 = 0.85;
pub const MAX: f64 = 50.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layout {
    pub crash: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub launched: Option<DateTime<Utc>>,
    /// The multiplier cashed out at
    pub cashed: Option<f64>,
    pub crashed: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Move {
    Launch,
    Cash,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    // P(crash > m) = HOUSE / m: inverse of a uniform draw
    let u: f64 = rng.random_range(0.0..1.0);
    let crash = (HOUSE / (1.0 - u)).clamp(1.0, MAX);
    let crash = (crash * 100.0).floor() / 100.0;
    Drawn { layout: serde_json::to_value(Layout { crash }).expect("serializes"), prize: Prize::Nothing }
}

pub fn multiplier(flown: Duration) -> f64 {
    let m = 2f64.powf(flown.num_milliseconds() as f64 / 1000.0 / DOUBLING);
    (m * 100.0).floor() / 100.0
}

/// When the rocket crashes, after launch
pub fn crash_after(crash: f64) -> Duration {
    Duration::milliseconds((crash.log2() * DOUBLING * 1000.0).ceil() as i64)
}

pub struct Rocket;

impl Game for Rocket {
    type Layout = Layout;
    type State = State;
    type Move = Move;

    fn view(l: &Layout, s: &State, now: DateTime<Utc>) -> serde_json::Value {
        let over = s.crashed || s.cashed.is_some();
        serde_json::json!({
            "launched": s.launched.is_some(),
            "ms_flown": s.launched.map(|t| (now - t).num_milliseconds()),
            "doubling": DOUBLING,
            "stake": STAKE,
            "cashed": s.cashed,
            "crashed": s.crashed,
            "crash": if over { Some(l.crash) } else { None },
        })
    }

    fn tick(l: &Layout, s: &mut State, now: DateTime<Utc>) -> Outcome {
        match s.launched {
            Some(t) if s.cashed.is_none() && !s.crashed && now - t >= crash_after(l.crash) => {
                s.crashed = true;
                Outcome::Over(Prize::Nothing)
            }
            _ => Outcome::Going,
        }
    }

    fn play(l: &Layout, s: &mut State, mv: Move, now: DateTime<Utc>) -> Result<Outcome, &'static str> {
        match (mv, s.launched) {
            (Move::Launch, None) => {
                s.launched = Some(now);
                // A rocket that crashes on the pad
                if l.crash <= 1.0 {
                    s.crashed = true;
                    return Ok(Outcome::Over(Prize::Nothing));
                }
                Ok(Outcome::Going)
            }
            (Move::Cash, Some(t)) => {
                let m = multiplier(now - t).min(l.crash);
                s.cashed = Some(m);
                Ok(Outcome::Over(Prize::Coins { amount: (STAKE as f64 * m).floor() as i64 }))
            }
            (Move::Launch, Some(_)) => Err("launched_already"),
            (Move::Cash, None) => Err("launch_first"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tickets::Series;
    use crate::tickets::play::dispatch;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn climbs_crashes_and_cashes() {
        assert_eq!(multiplier(Duration::seconds(4)), 2.0);
        assert_eq!(multiplier(Duration::seconds(8)), 4.0);
        let t0: DateTime<Utc> = "2026-11-06T10:00:00Z".parse().unwrap();
        let l = serde_json::json!({ "crash": 3.0 });
        let go = |state: Option<&serde_json::Value>, mv: &str, at| {
            dispatch(Series::Rocket, &l, state, Some(&serde_json::json!({ "kind": mv })), at).unwrap().unwrap()
        };
        let p = go(None, "launch", t0);
        assert!(p.view["crash"].is_null(), "the crash point stays hidden");
        let cashed = go(Some(&p.state), "cash", t0 + Duration::seconds(4));
        assert_eq!(cashed.outcome, Outcome::Over(Prize::Coins { amount: 200 }));
        // Past the crash, a cash out is too late: the look crashes it first
        let late = go(Some(&p.state), "cash", t0 + Duration::seconds(7));
        assert_eq!(late.outcome, Outcome::Over(Prize::Nothing));
        assert_eq!(late.view["crash"], 3.0);
    }

    /// Whatever multiplier a player aims for, the payback is the same, below the price
    #[test]
    fn every_target_pays_back_the_same() {
        let mut rng = StdRng::seed_from_u64(4);
        let n = 60_000;
        for target in [1.2, 2.0, 5.0] {
            let mut total = 0.0;
            for _ in 0..n {
                let l: Layout = serde_json::from_value(draw(&mut rng).layout).unwrap();
                if l.crash > target {
                    total += (STAKE as f64 * target).floor();
                }
            }
            let ratio = total / n as f64 / PRICE as f64;
            assert!((0.78..=0.9).contains(&ratio), "aiming for ×{target} pays back {ratio:.2}");
        }
    }
}
