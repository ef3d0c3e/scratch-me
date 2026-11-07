//! Roulette: three areas, red, black and green, each hiding a bet. The player
//! scratches one (their colour and the bet they placed), then the wheel spins:
//! 37 pockets, 18 red, 18 black and the green zero. Green's bets are as much
//! bigger as green is rarer, so every colour is worth the same.

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::play::{Game, Outcome};
use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 100;
pub const POCKETS: u8 = 37;

/// Red and black bets, and how often each comes up
pub const EVEN_BETS: [(i64, u32); 4] = [(140, 50), (170, 30), (220, 15), (400, 5)];
/// Green's: about 18 times as much, for a pocket in 37 instead of 18
pub const GREEN_BETS: [(i64, u32); 3] = [(2500, 60), (3500, 30), (5000, 10)];

/// The red pockets of a European wheel
const RED: [u8; 18] = [1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Colour {
    Red,
    Black,
    Green,
}

pub fn colour(pocket: u8) -> Colour {
    if pocket == 0 {
        Colour::Green
    } else if RED.contains(&pocket) {
        Colour::Red
    } else {
        Colour::Black
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// The bets under red, black and green
    pub bets: [i64; 3],
    /// Where the ball stops, 0 to 36
    pub pocket: u8,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub pick: Option<Colour>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Move {
    pub colour: Colour,
}

fn at(c: Colour) -> usize {
    match c {
        Colour::Red => 0,
        Colour::Black => 1,
        Colour::Green => 2,
    }
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let layout = Layout {
        bets: [*weighted(rng, &EVEN_BETS), *weighted(rng, &EVEN_BETS), *weighted(rng, &GREEN_BETS)],
        pocket: rng.random_range(0..POCKETS),
    };
    Drawn { layout: serde_json::to_value(layout).expect("serializes"), prize: Prize::Nothing }
}

pub fn prize_of(l: &Layout, pick: Colour) -> Prize {
    if colour(l.pocket) == pick { Prize::Coins { amount: l.bets[at(pick)] } } else { Prize::Nothing }
}

pub struct Roulette;

impl Game for Roulette {
    type Layout = Layout;
    type State = State;
    type Move = Move;

    fn view(l: &Layout, s: &State, _now: DateTime<Utc>) -> serde_json::Value {
        match s.pick {
            None => serde_json::json!({}),
            Some(pick) => serde_json::json!({
                "pick": pick, "bets": l.bets, "pocket": l.pocket, "colour": colour(l.pocket)
            }),
        }
    }

    fn play(l: &Layout, s: &mut State, mv: Move, _now: DateTime<Utc>) -> Result<Outcome, &'static str> {
        if s.pick.is_some() {
            return Err("bet_already");
        }
        s.pick = Some(mv.colour);
        Ok(Outcome::Over(prize_of(l, mv.colour)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn a_european_wheel() {
        let count = |c| (0..POCKETS).filter(|p| colour(*p) == c).count();
        assert_eq!((count(Colour::Red), count(Colour::Black), count(Colour::Green)), (18, 18, 1));
        let l = Layout { bets: [140, 170, 2500], pocket: 0 };
        assert_eq!(prize_of(&l, Colour::Green), Prize::Coins { amount: 2500 });
        assert_eq!(prize_of(&l, Colour::Red), Prize::Nothing);
        let mut s = State::default();
        let now = Utc::now();
        assert!(Roulette::play(&l, &mut s, Move { colour: Colour::Red }, now).is_ok());
        assert_eq!(Roulette::play(&l, &mut s, Move { colour: Colour::Green }, now), Err("bet_already"));
    }

    /// Every colour pays back 70–90 % of the price
    #[test]
    fn every_colour_pays_most_of_the_price() {
        let mut rng = StdRng::seed_from_u64(6);
        for pick in [Colour::Red, Colour::Black, Colour::Green] {
            let n = 400_000;
            let total: i64 = (0..n)
                .map(|_| {
                    let l: Layout = serde_json::from_value(draw(&mut rng).layout).unwrap();
                    prize_of(&l, pick).coins()
                })
                .sum();
            let ratio = total as f64 / n as f64 / PRICE as f64;
            assert!((0.70..=0.90).contains(&ratio), "{pick:?} pays back {ratio:.3}");
        }
    }
}
