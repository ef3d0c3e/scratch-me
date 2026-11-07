//! Levels from experience. Each level takes 100 more than the one before:
//! level 2 at 100 xp, 3 at 300, 4 at 600, 5 at 1,000…

use serde::Serialize;

use crate::tickets::{Prize, Series};

/// Experience at which `level` starts
pub fn xp_for(level: i32) -> i64 {
    let l = level.max(1) as i64;
    50 * l * (l - 1)
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

/// What reaching `level` gives: a Golden Ticket every fifth level, a Number Match otherwise
pub fn reward(level: i32) -> Prize {
    if level % 5 == 0 {
        Prize::Ticket { series: Series::Golden }
    } else {
        Prize::Ticket { series: Series::NumberMatch }
    }
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
        assert_eq!([xp_for(1), xp_for(2), xp_for(3), xp_for(5)], [0, 100, 300, 1000]);
        assert_eq!(level_for(0), 1);
        assert_eq!(level_for(99), 1);
        assert_eq!(level_for(100), 2);
        assert_eq!(level_for(1000), 5);
        for xp in [0, 57, 300, 4499, 4500, 123_456] {
            let d = dto(xp);
            assert!(d.from <= xp && xp < d.to);
        }
    }

    #[test]
    fn every_fifth_level_is_golden() {
        assert_eq!(reward(5), Prize::Ticket { series: Series::Golden });
        assert_eq!(reward(10), Prize::Ticket { series: Series::Golden });
        assert_eq!(reward(2), Prize::Ticket { series: Series::NumberMatch });
    }
}
