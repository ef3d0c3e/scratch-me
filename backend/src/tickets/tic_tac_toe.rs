//! Tic-Tac-Toe: three games of noughts and crosses, played out on full
//! boards. You are X: a line of three X wins that game's prize.

use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 35;
pub const GAMES: usize = 3;
/// Chance out of 1,000 that a game is won
const WIN: u32 = 250;

pub const LABELS: [(i64, u32); 5] = [(10, 400), (20, 300), (50, 200), (100, 80), (500, 20)];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mark {
    X,
    O,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    /// Row by row
    pub board: Vec<Mark>,
    pub coins: i64,
    pub won: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub games: Vec<Game>,
}

const LINES: [[usize; 3]; 8] = [[0, 1, 2], [3, 4, 5], [6, 7, 8], [0, 3, 6], [1, 4, 7], [2, 5, 8], [0, 4, 8], [2, 4, 6]];

pub fn has_line(board: &[Mark], m: Mark) -> bool {
    LINES.iter().any(|l| l.iter().all(|&i| board[i] == m))
}

/// A full board (five X, four O) that is a win for X or not, as asked
fn board(win: bool, rng: &mut impl Rng) -> Vec<Mark> {
    loop {
        let mut b = vec![Mark::X; 5];
        b.extend([Mark::O; 4]);
        b.shuffle(rng);
        if has_line(&b, Mark::X) == win {
            return b;
        }
    }
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let games: Vec<Game> = (0..GAMES)
        .map(|_| {
            let won = rng.random_range(0..1000) < WIN;
            Game { board: board(won, rng), coins: *weighted(rng, &LABELS), won }
        })
        .collect();
    let total: i64 = games.iter().filter(|g| g.won).map(|g| g.coins).sum();
    let prize = if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing };
    Drawn { layout: serde_json::to_value(Layout { games }).expect("serializes"), prize }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn boards_show_their_result() {
        let mut rng = StdRng::seed_from_u64(21);
        for _ in 0..2000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            let mut total = 0;
            for g in &l.games {
                assert_eq!(g.board.len(), 9);
                assert_eq!(g.board.iter().filter(|m| **m == Mark::X).count(), 5);
                assert_eq!(has_line(&g.board, Mark::X), g.won);
                if g.won {
                    total += g.coins;
                }
            }
            assert_eq!(d.prize, if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing });
        }
    }
}
