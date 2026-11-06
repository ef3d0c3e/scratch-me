//! Duel: two players, three dice each, the higher total wins the pot (both
//! stakes, less what the house keeps). A tie rolls again. Each player gets a
//! ticket showing the same duel from their side.

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize};

pub const STAKE: i64 = 100;
/// The house keeps 10 % of the pot
pub const POT: i64 = STAKE * 2 * 9 / 10;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub opponent: String,
    pub mine: Vec<u8>,
    pub theirs: Vec<u8>,
    pub won: bool,
}

/// Rolls until the totals differ: (challenger's dice, opponent's dice)
pub fn roll(rng: &mut impl Rng) -> (Vec<u8>, Vec<u8>) {
    loop {
        let a: Vec<u8> = (0..3).map(|_| rng.random_range(1..=6)).collect();
        let b: Vec<u8> = (0..3).map(|_| rng.random_range(1..=6)).collect();
        if a.iter().sum::<u8>() != b.iter().sum::<u8>() {
            return (a, b);
        }
    }
}

/// One player's ticket for a duel
pub fn ticket(opponent: &str, mine: &[u8], theirs: &[u8]) -> Drawn {
    let won = mine.iter().sum::<u8>() > theirs.iter().sum::<u8>();
    let layout = Layout { opponent: opponent.to_owned(), mine: mine.to_vec(), theirs: theirs.to_vec(), won };
    Drawn {
        layout: serde_json::to_value(layout).expect("serializes"),
        prize: if won { Prize::Coins { amount: POT } } else { Prize::Nothing },
    }
}

/// Never drawn on its own: duels are made by `routes/duels.rs`
pub fn draw(_rng: &mut impl Rng) -> Drawn {
    Drawn { layout: serde_json::json!({}), prize: Prize::Nothing }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_wins_one_loses() {
        let mut rng = rand::rng();
        for _ in 0..500 {
            let (a, b) = roll(&mut rng);
            let (ta, tb) = (ticket("bob", &a, &b), ticket("ana", &b, &a));
            assert_ne!(ta.prize == Prize::Nothing, tb.prize == Prize::Nothing, "exactly one winner");
            assert_eq!(ta.prize.coins() + tb.prize.coins(), POT);
        }
        const { assert!(POT < 2 * STAKE, "the house keeps a share") };
    }
}
