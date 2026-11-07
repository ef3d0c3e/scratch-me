//! Keno: twenty numbers, eight of them drawn when the ticket is issued. The
//! player scratches five as their picks, each showing at once whether it was
//! drawn; the fifth ends it, paid by how many matched. The draw is hidden,
//! so every pick is worth the same.

use chrono::{DateTime, Utc};
use rand::Rng;
use rand::seq::index;
use serde::{Deserialize, Serialize};

use super::play::{Game, Outcome};
use super::{Drawn, Prize};

pub const PRICE: i64 = 80;
pub const NUMBERS: usize = 20;
pub const DRAWN: usize = 8;
pub const PICKS: usize = 5;
/// What each count of matches pays
pub const PAYS: [i64; PICKS + 1] = [0, 0, 30, 100, 400, 2500];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// The numbers drawn, as cells 0 to 19 (shown as 1 to 20)
    pub drawn: Vec<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub picks: Vec<usize>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Move {
    pub pick: usize,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let mut drawn = index::sample(rng, NUMBERS, DRAWN).into_vec();
    drawn.sort();
    Drawn { layout: serde_json::to_value(Layout { drawn }).expect("serializes"), prize: Prize::Nothing }
}

fn matches(l: &Layout, picks: &[usize]) -> usize {
    picks.iter().filter(|p| l.drawn.contains(p)).count()
}

pub fn prize_of(l: &Layout, picks: &[usize]) -> Prize {
    let coins = PAYS[matches(l, picks).min(PICKS)];
    if coins > 0 { Prize::Coins { amount: coins } } else { Prize::Nothing }
}

pub struct Keno;

impl Game for Keno {
    type Layout = Layout;
    type State = State;
    type Move = Move;

    fn view(l: &Layout, s: &State, _now: DateTime<Utc>) -> serde_json::Value {
        let over = s.picks.len() >= PICKS;
        serde_json::json!({
            "picks": s.picks.iter().map(|p| serde_json::json!({ "cell": p, "hit": l.drawn.contains(p) })).collect::<Vec<_>>(),
            "matches": matches(l, &s.picks),
            "left": PICKS - s.picks.len().min(PICKS),
            "pays": PAYS,
            "drawn": if over { Some(&l.drawn) } else { None },
        })
    }

    fn play(l: &Layout, s: &mut State, mv: Move, _now: DateTime<Utc>) -> Result<Outcome, &'static str> {
        if mv.pick >= NUMBERS {
            return Err("no_such_cell");
        }
        if s.picks.contains(&mv.pick) {
            return Err("picked_already");
        }
        s.picks.push(mv.pick);
        Ok(if s.picks.len() >= PICKS { Outcome::Over(prize_of(l, &s.picks)) } else { Outcome::Going })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn five_picks_paid_by_their_matches() {
        let l = Layout { drawn: vec![0, 1, 2, 3, 4, 5, 6, 7] };
        assert_eq!(prize_of(&l, &[0, 1, 2, 10, 11]), Prize::Coins { amount: 100 });
        assert_eq!(prize_of(&l, &[0, 10, 11, 12, 13]), Prize::Nothing);
        let mut s = State::default();
        let now = Utc::now();
        for p in [0, 1, 2, 3] {
            assert_eq!(Keno::play(&l, &mut s, Move { pick: p }, now), Ok(Outcome::Going));
        }
        assert_eq!(Keno::play(&l, &mut s, Move { pick: 3 }, now), Err("picked_already"));
        assert!(Keno::view(&l, &s, now)["drawn"].is_null(), "the draw stays hidden");
        assert_eq!(Keno::play(&l, &mut s, Move { pick: 4 }, now), Ok(Outcome::Over(Prize::Coins { amount: 2500 })));
    }

    #[test]
    fn picks_pay_back_most_of_the_price() {
        let mut rng = StdRng::seed_from_u64(2);
        let n = 300_000;
        let total: i64 = (0..n)
            .map(|_| {
                let l: Layout = serde_json::from_value(draw(&mut rng).layout).unwrap();
                prize_of(&l, &[0, 5, 9, 14, 19]).coins()
            })
            .sum();
        let ratio = total as f64 / n as f64 / PRICE as f64;
        assert!((0.70..=0.90).contains(&ratio), "keno pays back {ratio:.3}");
    }
}
