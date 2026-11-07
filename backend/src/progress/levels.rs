//! Levels from experience. Each level takes STEP more than the one before:
//! level 2 at STEP xp, 3 at 3 × STEP, 4 at 6 × STEP… (`progress/pace.rs`
//! checks what a first day reaches).

use serde::Serialize;

use crate::tickets::{Prize, Series};

/// What level 2 takes, and each next level that much more than the last
pub const STEP: i64 = 300;

/// Experience at which `level` starts
pub fn xp_for(level: i32) -> i64 {
    let l = level.max(1) as i64;
    STEP / 2 * l * (l - 1)
}

pub fn level_for(xp: i64) -> i32 {
    let mut level = 1;
    while xp_for(level + 1) <= xp {
        level += 1;
    }
    level
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LevelDto {
    pub level: i32,
    pub xp: i64,
    /// Where this level started and the next begins
    pub from: i64,
    pub to: i64,
}

pub fn dto(xp: i64) -> LevelDto {
    let level = level_for(xp);
    LevelDto { level, xp, from: xp_for(level), to: xp_for(level + 1) }
}

/// What reaching a level gives: a Ticket Chain, always
pub fn reward(_level: i32) -> Prize {
    Prize::Ticket { series: Series::TicketChain }
}

/// The level each shop tier opens at
pub const TIER_LEVELS: [i32; 3] = [1, 3, 6];

/// The level a series needs in the shop: its tier's
pub fn shop_level(series: Series) -> i32 {
    series.tier().map_or(1, |t| TIER_LEVELS[t as usize - 1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve() {
        assert_eq!([xp_for(1), xp_for(2), xp_for(3), xp_for(5)], [0, 300, 900, 3000]);
        assert_eq!(level_for(0), 1);
        assert_eq!(level_for(299), 1);
        assert_eq!(level_for(300), 2);
        assert_eq!(level_for(3000), 5);
        assert_eq!(level_for(xp_for(50)), 50);
        for xp in [0, 57, 300, 4499, 4500, 123_456] {
            let d = dto(xp);
            assert!(d.from <= xp && xp < d.to);
        }
    }

    #[test]
    fn every_level_gives_a_chain() {
        for l in [2, 5, 10, 50] {
            assert_eq!(reward(l), Prize::Ticket { series: Series::TicketChain });
        }
    }
}
