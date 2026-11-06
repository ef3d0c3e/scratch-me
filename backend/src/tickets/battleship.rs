//! Battleship: a 5×5 sea with three ships, and ten shots of yours. Every hit
//! pays a little; every ship hit on all its cells pays its prize.

use rand::Rng;
use rand::seq::index;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize};

pub const PRICE: i64 = 75;
pub const SIZE: usize = 5;
pub const SHOTS: usize = 10;
pub const HIT: i64 = 3;
/// Ship lengths and what sinking each pays
pub const SHIPS: [(usize, i64); 3] = [(3, 400), (2, 100), (2, 100)];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ship {
    /// Cells as row * 5 + column
    pub cells: Vec<usize>,
    pub coins: i64,
    pub sunk: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub ships: Vec<Ship>,
    pub shots: Vec<usize>,
    pub hits: usize,
}

/// Ships placed in a line each, not touching each other's cells
fn place(rng: &mut impl Rng) -> Vec<Vec<usize>> {
    'again: loop {
        let mut taken: Vec<usize> = vec![];
        let mut out = vec![];
        for (len, _) in SHIPS {
            let mut placed = None;
            for _ in 0..50 {
                let across = rng.random_bool(0.5);
                let (r, c) = if across {
                    (rng.random_range(0..SIZE), rng.random_range(0..=SIZE - len))
                } else {
                    (rng.random_range(0..=SIZE - len), rng.random_range(0..SIZE))
                };
                let cells: Vec<usize> =
                    (0..len).map(|i| if across { r * SIZE + c + i } else { (r + i) * SIZE + c }).collect();
                if cells.iter().all(|x| !taken.contains(x)) {
                    placed = Some(cells);
                    break;
                }
            }
            let Some(cells) = placed else { continue 'again };
            taken.extend(&cells);
            out.push(cells);
        }
        return out;
    }
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let shots: Vec<usize> = index::sample(rng, SIZE * SIZE, SHOTS).into_vec();
    let ships: Vec<Ship> = place(rng)
        .into_iter()
        .zip(SHIPS)
        .map(|(cells, (_, coins))| {
            let sunk = cells.iter().all(|c| shots.contains(c));
            Ship { cells, coins, sunk }
        })
        .collect();
    let hits = ships.iter().flat_map(|s| &s.cells).filter(|c| shots.contains(c)).count();
    let total = hits as i64 * HIT + ships.iter().filter(|s| s.sunk).map(|s| s.coins).sum::<i64>();
    let prize = if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing };
    Drawn { layout: serde_json::to_value(Layout { ships, shots, hits }).expect("serializes"), prize }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hits_and_sinkings_are_paid() {
        let mut rng = rand::rng();
        for _ in 0..2000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            let mut all: Vec<usize> = l.ships.iter().flat_map(|s| s.cells.clone()).collect();
            all.sort();
            all.dedup();
            assert_eq!(all.len(), 7, "ships do not overlap");
            assert!(all.iter().all(|c| *c < 25));
            let sunk: i64 = l.ships.iter().filter(|s| s.sunk).map(|s| s.coins).sum();
            assert_eq!(d.prize.coins(), l.hits as i64 * HIT + sunk);
        }
    }
}
