//! Seals: what a ticket from a bundle carries. A seal is a theme (how the
//! ticket looks) and effects on what it pays: a multiplier on its coins, a
//! sure win, a second chance. It is what lets a dear bundle hold cheap
//! tickets that pay like dear ones.
//!
//! Sure wins, second chances and best-of draws act when the ticket is drawn;
//! insurance and echoes when it is finished, but are only valued (and so only
//! sold) on tickets drawn whole: none of them go on tickets played move by
//! move. The multiplier acts when the ticket is finished, on any ticket.

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
    Fortune,
    Safety,
    Echo,
    Halloween,
    Winter,
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
    /// Drawn this many times, the best kept (1: once)
    pub best: u8,
    /// A losing ticket pays back this share of its price, in percent
    pub insurance: i64,
    /// A winning ticket also gives a free one of its kind
    pub echo: bool,
}

const PLAIN: Seal =
    Seal { key: "", theme: Theme::Sunrise, times: 1, sure: false, second: false, best: 1, insurance: 0, echo: false };

pub const SEALS: [Seal; 10] = [
    Seal { key: "sunrise", theme: Theme::Sunrise, second: true, ..PLAIN },
    Seal { key: "beach", theme: Theme::Beach, times: 2, ..PLAIN },
    Seal { key: "casino", theme: Theme::Casino, times: 3, ..PLAIN },
    Seal { key: "diamond", theme: Theme::Diamond, times: 10, sure: true, ..PLAIN },
    Seal { key: "cosmos", theme: Theme::Cosmos, times: 3, ..PLAIN },
    Seal { key: "fortune", theme: Theme::Fortune, best: 3, ..PLAIN },
    Seal { key: "safety", theme: Theme::Safety, insurance: 50, ..PLAIN },
    Seal { key: "echo", theme: Theme::Echo, echo: true, ..PLAIN },
    Seal { key: "halloween", theme: Theme::Halloween, times: 3, ..PLAIN },
    Seal { key: "winter", theme: Theme::Winter, times: 2, second: true, ..PLAIN },
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
        let needs_whole = self.sure || self.second || self.best > 1 || self.insurance > 0 || self.echo;
        !needs_whole || redrawable(series)
    }

    /// Draws a ticket of `series` under this seal
    pub fn draw(self, series: Series, rng: &mut impl Rng) -> Drawn {
        let mut drawn = series.draw(rng);
        if self.best > 1 {
            for _ in 1..self.best {
                let other = series.draw(rng);
                if worth(&other.prize) > worth(&drawn.prize) {
                    drawn = other;
                }
            }
        } else if self.sure {
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

    /// Coins the seal adds to a finished ticket's prize: its multiplier, and
    /// its insurance on a loss (a share of `price`, what the ticket costs)
    pub fn bonus(self, prize: &Prize, price: i64) -> i64 {
        let refund = if *prize == Prize::Nothing { price * self.insurance / 100 } else { 0 };
        prize.coins() * (self.times - 1) + refund
    }
}

/// How a draw ranks for a best-of seal: its coins, a ticket counted at a
/// typical ticket's worth
fn worth(prize: &Prize) -> i64 {
    match prize {
        Prize::Nothing => 0,
        Prize::Coins { amount } => *amount,
        Prize::Ticket { series } => series.price().unwrap_or(100),
        Prize::Tickets { series } => series.iter().map(|s| s.price().unwrap_or(100)).sum(),
        Prize::PocketRefill => 60,
        Prize::Cosmetic { .. } => crate::cosmetics::DUPLICATE_COINS,
        Prize::Bundle { items } => items.iter().map(worth).sum(),
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
        assert_eq!(casino.bonus(&Prize::Coins { amount: 50 }, 100), 100);
        assert_eq!(casino.bonus(&Prize::Ticket { series: Series::LuckyThree }, 100), 0);
        assert_eq!(seal("sunrise").unwrap().bonus(&Prize::Coins { amount: 50 }, 100), 0);
        let safety = seal("safety").unwrap();
        assert_eq!(safety.bonus(&Prize::Nothing, 140), 70, "half the price back on a loss");
        assert_eq!(safety.bonus(&Prize::Coins { amount: 10 }, 140), 0);
    }

    #[test]
    fn best_of_three_wins_more_often() {
        let mut rng = rand::rng();
        let fortune = seal("fortune").unwrap();
        let wins = |s: Seal, rng: &mut _| {
            (0..4000).filter(|_| s.draw(Series::LuckyThree, rng).prize != Prize::Nothing).count()
        };
        let (best, plain) = (wins(fortune, &mut rng), wins(PLAIN, &mut rng));
        assert!(best as f64 > plain as f64 * 1.8, "{best} wins of 4000 against {plain}");
        assert!(!fortune.fits(Series::Keno) && !seal("echo").unwrap().fits(Series::Rocket));
    }
}
