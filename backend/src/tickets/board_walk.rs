//! Board Walk: three dice rolls move a token round a sixteen-square board;
//! every square it lands on pays what it shows. The board is the same on
//! every ticket, so players learn where the big square is.

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize};

pub const PRICE: i64 = 40;
pub const ROLLS: usize = 3;
/// Clockwise from the start square
pub const BOARD: [i64; 16] = [0, 5, 0, 10, 0, 0, 20, 5, 0, 40, 0, 0, 10, 5, 0, 120];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub rolls: Vec<u8>,
    /// Squares landed on, in order
    pub path: Vec<usize>,
    pub board: Vec<i64>,
}

pub fn walk(rolls: &[u8]) -> Vec<usize> {
    let mut at = 0;
    rolls
        .iter()
        .map(|r| {
            at = (at + *r as usize) % BOARD.len();
            at
        })
        .collect()
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let rolls: Vec<u8> = (0..ROLLS).map(|_| rng.random_range(1..=6)).collect();
    let path = walk(&rolls);
    let total: i64 = path.iter().map(|&s| BOARD[s]).sum();
    let prize = if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing };
    Drawn { layout: serde_json::to_value(Layout { rolls, path, board: BOARD.to_vec() }).expect("serializes"), prize }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_token_walks_and_collects() {
        assert_eq!(walk(&[6, 3, 6]), vec![6, 9, 15]);
        assert_eq!(walk(&[6, 6, 6]), vec![6, 12, 2], "round the corner");
        let mut rng = rand::rng();
        for _ in 0..1000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            assert_eq!(l.path, walk(&l.rolls));
            assert_eq!(d.prize.coins(), l.path.iter().map(|&s| BOARD[s]).sum::<i64>());
        }
    }
}
