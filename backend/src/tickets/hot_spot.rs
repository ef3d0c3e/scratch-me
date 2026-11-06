//! Hot Spot: ten numbers of yours, twenty drawn out of eighty, paid by how
//! many match (like keno).

use rand::Rng;
use rand::seq::index;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize};

pub const PRICE: i64 = 65;
pub const MAX: u8 = 80;
pub const MINE: usize = 10;
pub const DRAWN: usize = 20;
/// What 0 to 10 matches pay
pub const PAYS: [i64; MINE + 1] = [0, 0, 0, 20, 80, 250, 1000, 4000, 15_000, 50_000, 100_000];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub mine: Vec<u8>,
    pub drawn: Vec<u8>,
    pub matches: usize,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let pick =
        |rng: &mut _, n| -> Vec<u8> { index::sample(rng, MAX as usize, n).into_iter().map(|i| i as u8 + 1).collect() };
    let mine = pick(rng, MINE);
    let drawn = pick(rng, DRAWN);
    let matches = mine.iter().filter(|n| drawn.contains(n)).count();
    let prize = if PAYS[matches] > 0 { Prize::Coins { amount: PAYS[matches] } } else { Prize::Nothing };
    Drawn { layout: serde_json::to_value(Layout { mine, drawn, matches }).expect("serializes"), prize }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_are_counted() {
        let mut rng = rand::rng();
        for _ in 0..2000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            assert_eq!(l.mine.len(), MINE);
            assert_eq!(l.drawn.len(), DRAWN);
            assert_eq!(l.matches, l.mine.iter().filter(|n| l.drawn.contains(n)).count());
            assert_eq!(d.prize.coins(), PAYS[l.matches]);
        }
    }
}
