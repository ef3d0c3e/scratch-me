//! Treasure Dig: a 4×4 beach and four digs, where the player chooses. Chests
//! hold coins; a map gives one more dig. The beach is laid out when the ticket
//! is issued and the server keeps it: the player learns a spot only by digging
//! it, and the prize is what their digs found.

use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 300;
pub const CELLS: usize = 16;
pub const DIGS: usize = 4;
const CHESTS: usize = 3;

pub const CHEST_COINS: [(i64, u32); 5] = [(100, 40), (200, 30), (400, 20), (1000, 8), (3000, 2)];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Spot {
    Sand,
    Chest {
        coins: i64,
    },
    /// One more dig
    Map,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub cells: Vec<Spot>,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let mut cells = vec![Spot::Sand; CELLS];
    for cell in cells.iter_mut().take(CHESTS) {
        *cell = Spot::Chest { coins: *weighted(rng, &CHEST_COINS) };
    }
    cells[CHESTS] = Spot::Map;
    cells.shuffle(rng);
    // The prize comes from the digs: none until the last one
    Drawn { layout: serde_json::to_value(Layout { cells }).expect("serializes"), prize: Prize::Nothing }
}

/// Digs allowed with these picks: one more for each map found
pub fn digs_allowed(layout: &Layout, picks: &[usize]) -> usize {
    DIGS + picks.iter().filter(|&&p| layout.cells[p] == Spot::Map).count()
}

/// What a dig asks: refused when the spot is off the beach, dug already, or no dig is left
pub fn can_dig(layout: &Layout, picks: &[usize], cell: usize) -> Result<(), &'static str> {
    if cell >= CELLS {
        return Err("no_such_spot");
    }
    if picks.contains(&cell) {
        return Err("dug_already");
    }
    if picks.len() >= digs_allowed(layout, picks) {
        return Err("no_digs_left");
    }
    Ok(())
}

pub fn finished(layout: &Layout, picks: &[usize]) -> bool {
    picks.len() >= digs_allowed(layout, picks) || picks.len() == CELLS
}

/// What the digs found
pub fn prize(layout: &Layout, picks: &[usize]) -> Prize {
    let total: i64 = picks
        .iter()
        .map(|&p| match layout.cells[p] {
            Spot::Chest { coins } => coins,
            _ => 0,
        })
        .sum();
    if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing }
}

/// Plays a ticket with random digs, as a player who knows nothing would
pub fn play_blind(layout: &Layout, rng: &mut impl Rng) -> Prize {
    let mut order: Vec<usize> = (0..CELLS).collect();
    order.shuffle(rng);
    let mut picks = vec![];
    for cell in order {
        if finished(layout, &picks) {
            break;
        }
        picks.push(cell);
    }
    prize(layout, &picks)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn beach() -> Layout {
        let mut cells = vec![Spot::Sand; CELLS];
        cells[0] = Spot::Chest { coins: 400 };
        cells[1] = Spot::Map;
        cells[2] = Spot::Chest { coins: 100 };
        Layout { cells }
    }

    #[test]
    fn four_digs_and_one_more_per_map() {
        let b = beach();
        assert_eq!(can_dig(&b, &[], 0), Ok(()));
        assert_eq!(can_dig(&b, &[0], 0), Err("dug_already"));
        assert_eq!(can_dig(&b, &[], 16), Err("no_such_spot"));
        assert!(!finished(&b, &[5, 6, 7]));
        assert!(finished(&b, &[5, 6, 7, 8]));
        assert_eq!(can_dig(&b, &[5, 6, 7, 8], 9), Err("no_digs_left"));
        // The map: a fifth dig
        assert!(!finished(&b, &[1, 6, 7, 8]));
        assert_eq!(can_dig(&b, &[1, 6, 7, 8], 0), Ok(()));
        assert!(finished(&b, &[1, 6, 7, 8, 0]));
    }

    #[test]
    fn the_prize_is_what_was_dug() {
        let b = beach();
        assert_eq!(prize(&b, &[0, 2, 5, 6]), Prize::Coins { amount: 500 });
        assert_eq!(prize(&b, &[5, 6, 7, 8]), Prize::Nothing);
    }

    #[test]
    fn every_beach_has_its_chests_and_a_map() {
        let mut rng = rand::rng();
        for _ in 0..500 {
            let l: Layout = serde_json::from_value(draw(&mut rng).layout).unwrap();
            assert_eq!(l.cells.iter().filter(|c| matches!(c, Spot::Chest { .. })).count(), CHESTS);
            assert_eq!(l.cells.iter().filter(|c| **c == Spot::Map).count(), 1);
        }
    }
}
