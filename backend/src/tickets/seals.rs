//! Seals: what a ticket from a bundle carries. A seal is a theme (how the
//! ticket looks) and effects on what it pays: a multiplier on its coins, a
//! sure win, a second chance. It is what lets a dear bundle hold cheap
//! tickets that pay like dear ones.
//!
//! Sure wins and second chances act when the ticket is drawn, so they only
//! go on tickets drawn whole (never on those played move by move). The
//! multiplier acts when the ticket is finished, on any ticket.

use rand::Rng;
use serde::Serialize;

use super::{Drawn, Prize, Series};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Sunrise,
    Beach,
    Casino,
    Diamond,
    Cosmos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Seal {
    pub key: &'static str,
    pub theme: Theme,
    /// Coins paid are multiplied by this (1: unchanged)
    pub times: i64,
    /// Drawn again until it wins
    pub sure: bool,
    /// A losing draw is drawn once more
    pub second: bool,
}

pub const SEALS: [Seal; 5] = [
    Seal { key: "sunrise", theme: Theme::Sunrise, times: 1, sure: false, second: true },
    Seal { key: "beach", theme: Theme::Beach, times: 2, sure: false, second: false },
    Seal { key: "casino", theme: Theme::Casino, times: 3, sure: false, second: false },
    Seal { key: "diamond", theme: Theme::Diamond, times: 10, sure: true, second: false },
    Seal { key: "cosmos", theme: Theme::Cosmos, times: 3, sure: false, second: false },
];

pub fn seal(key: &str) -> Option<Seal> {
    SEALS.into_iter().find(|s| s.key == key)
}

/// A draw can be redone: the ticket's whole outcome is in the draw
pub fn redrawable(series: Series) -> bool {
    !series.is_played() && !series.is_dig()
}

impl Seal {
    /// Whether this seal can go on a ticket of this series
    pub fn fits(self, series: Series) -> bool {
        !(self.sure || self.second) || redrawable(series)
    }

    /// Draws a ticket of `series` under this seal
    pub fn draw(self, series: Series, rng: &mut impl Rng) -> Drawn {
        let mut drawn = series.draw(rng);
        if self.sure {
            // A win comes within a few draws for every series; capped all the same
            for _ in 0..200 {
                if drawn.prize != Prize::Nothing {
                    break;
                }
                drawn = series.draw(rng);
            }
        } else if self.second && drawn.prize == Prize::Nothing {
            drawn = series.draw(rng);
        }
        drawn
    }

    /// Coins the seal adds to a finished ticket's prize
    pub fn bonus(self, prize: &Prize) -> i64 {
        prize.coins() * (self.times - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_unique_and_found() {
        for s in SEALS {
            assert_eq!(seal(s.key), Some(s));
            assert!(s.times >= 1);
        }
        assert_eq!(seal("nope"), None);
    }

    #[test]
    fn sure_wins_always_win() {
        let mut rng = rand::rng();
        let diamond = seal("diamond").unwrap();
        assert!((0..2000).all(|_| diamond.draw(Series::LuckyThree, &mut rng).prize != Prize::Nothing));
        assert!(!diamond.fits(Series::Rocket), "played tickets cannot be drawn again");
        assert!(seal("beach").unwrap().fits(Series::Battleship), "a multiplier fits anything");
    }

    #[test]
    fn multipliers_add_to_coins_only() {
        let casino = seal("casino").unwrap();
        assert_eq!(casino.bonus(&Prize::Coins { amount: 50 }), 100);
        assert_eq!(casino.bonus(&Prize::Ticket { series: Series::LuckyThree }), 0);
        assert_eq!(seal("sunrise").unwrap().bonus(&Prize::Coins { amount: 50 }), 0);
    }
}
