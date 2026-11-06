//! Lucky Dice: three dice. A triple pays most (sixes most of all), then a
//! straight (three in a row), then a pair.

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize};

pub const PRICE: i64 = 45;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Roll {
    Sixes,
    Triple,
    Straight,
    Pair,
    Nothing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub dice: Vec<u8>,
    pub roll: Roll,
}

pub fn roll(dice: &[u8]) -> Roll {
    let mut d = dice.to_vec();
    d.sort();
    if d[0] == d[2] {
        if d[0] == 6 { Roll::Sixes } else { Roll::Triple }
    } else if d[1] == d[0] + 1 && d[2] == d[1] + 1 {
        Roll::Straight
    } else if d[0] == d[1] || d[1] == d[2] {
        Roll::Pair
    } else {
        Roll::Nothing
    }
}

pub fn pays(r: Roll) -> i64 {
    match r {
        Roll::Sixes => 2000,
        Roll::Triple => 300,
        Roll::Straight => 100,
        Roll::Pair => 20,
        Roll::Nothing => 0,
    }
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let dice: Vec<u8> = (0..3).map(|_| rng.random_range(1..=6)).collect();
    let r = roll(&dice);
    let prize = if pays(r) > 0 { Prize::Coins { amount: pays(r) } } else { Prize::Nothing };
    Drawn { layout: serde_json::to_value(Layout { dice, roll: r }).expect("serializes"), prize }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolls() {
        assert_eq!(roll(&[6, 6, 6]), Roll::Sixes);
        assert_eq!(roll(&[2, 2, 2]), Roll::Triple);
        assert_eq!(roll(&[4, 2, 3]), Roll::Straight);
        assert_eq!(roll(&[5, 1, 5]), Roll::Pair);
        assert_eq!(roll(&[1, 3, 6]), Roll::Nothing);
    }
}
