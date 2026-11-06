//! Tickets: series, prizes, and the draw. Rules only, no I/O: routes store the
//! drawn ticket and apply its prize.
//!
//! A ticket's outcome is drawn when it is issued. The player only gets the
//! layout when they scratch it.

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::cosmetics::Cosmetic;

pub mod cash_wheel;
pub mod golden;
pub mod lucky_three;
pub mod number_match;
pub mod ticket_chain;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Series {
    LuckyThree,
    NumberMatch,
    TicketChain,
    CashWheel,
    Golden,
}

impl Series {
    pub const ALL: [Series; 5] =
        [Series::LuckyThree, Series::NumberMatch, Series::TicketChain, Series::CashWheel, Series::Golden];

    /// The name stored in the database and sent to the client
    pub fn key(self) -> &'static str {
        match self {
            Series::LuckyThree => "lucky_three",
            Series::NumberMatch => "number_match",
            Series::TicketChain => "ticket_chain",
            Series::CashWheel => "cash_wheel",
            Series::Golden => "golden",
        }
    }

    pub fn parse(key: &str) -> Option<Series> {
        Series::ALL.into_iter().find(|s| s.key() == key)
    }

    /// Shop price, None for series that only come as prizes and drops
    pub fn price(self) -> Option<i64> {
        match self {
            Series::LuckyThree => Some(lucky_three::PRICE),
            Series::NumberMatch => Some(number_match::PRICE),
            Series::CashWheel => Some(cash_wheel::PRICE),
            Series::TicketChain | Series::Golden => None,
        }
    }

    /// Draws a ticket of this series
    pub fn draw(self, rng: &mut impl Rng) -> Drawn {
        match self {
            Series::LuckyThree => lucky_three::draw(rng),
            Series::NumberMatch => number_match::draw(rng),
            Series::TicketChain => ticket_chain::draw(rng),
            Series::CashWheel => cash_wheel::draw(rng),
            Series::Golden => golden::draw(rng),
        }
    }
}

/// What scratching a ticket gives
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Prize {
    Nothing,
    Coins {
        amount: i64,
    },
    /// Another ticket, issued when this one is scratched
    Ticket {
        series: Series,
    },
    /// Several tickets at once
    Tickets {
        series: Vec<Series>,
    },
    /// The pocket is filled up at once
    PocketRefill,
    /// A theme or a scratch tool; coins instead when already owned
    Cosmetic {
        item: Cosmetic,
    },
}

/// A drawn ticket: what the player will uncover and what it pays
#[derive(Debug, Clone)]
pub struct Drawn {
    pub layout: serde_json::Value,
    pub prize: Prize,
}

/// Where a ticket came from
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Pocket,
    Envelope,
    Prize,
    Shop,
    /// Levels, quests and achievements
    Reward,
}

impl Source {
    pub fn key(self) -> &'static str {
        match self {
            Source::Pocket => "pocket",
            Source::Envelope => "envelope",
            Source::Prize => "prize",
            Source::Shop => "shop",
            Source::Reward => "reward",
        }
    }
}

/// Picks from (item, weight) pairs; weights need not sum to anything
pub(crate) fn weighted<'a, T>(rng: &mut impl Rng, table: &'a [(T, u32)]) -> &'a T {
    let total: u32 = table.iter().map(|(_, w)| w).sum();
    let mut roll = rng.random_range(0..total);
    for (item, w) in table {
        if roll < *w {
            return item;
        }
        roll -= w;
    }
    &table[table.len() - 1].0
}

/// The expected value in coins of a ticket of `series`, measured by drawing:
/// what tests compare to prices. Tickets won count at their own value.
#[cfg(test)]
pub(crate) fn measured_value(series: Series, n: usize, rng: &mut impl Rng) -> f64 {
    fn value(prize: &Prize, depth: u32, rng: &mut impl Rng) -> f64 {
        let ticket = |s: Series, rng: &mut _| {
            if depth > 6 { 0.0 } else { value(&s.draw(rng).prize, depth + 1, rng) }
        };
        match prize {
            Prize::Nothing | Prize::PocketRefill => 0.0,
            Prize::Coins { amount } => *amount as f64,
            Prize::Ticket { series } => ticket(*series, rng),
            Prize::Tickets { series } => series.iter().map(|s| ticket(*s, rng)).sum(),
            // Valued at what they pay as a duplicate
            Prize::Cosmetic { .. } => crate::cosmetics::DUPLICATE_COINS as f64,
        }
    }
    (0..n).map(|_| value(&series.draw(rng).prize, 0, rng)).sum::<f64>() / n as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn series_keys_round_trip() {
        for s in Series::ALL {
            assert_eq!(Series::parse(s.key()), Some(s));
            assert_eq!(serde_json::to_value(s).unwrap(), s.key());
        }
        assert_eq!(Series::parse("nope"), None);
    }

    #[test]
    fn prize_json() {
        let p = Prize::Ticket { series: Series::LuckyThree };
        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(json, serde_json::json!({ "kind": "ticket", "series": "lucky_three" }));
        assert_eq!(serde_json::from_value::<Prize>(json).unwrap(), p);
        let c = Prize::Cosmetic { item: Cosmetic::Theme(crate::cosmetics::Theme::Neon) };
        assert_eq!(
            serde_json::to_value(&c).unwrap(),
            serde_json::json!({ "kind": "cosmetic", "item": { "kind": "theme", "id": "neon" } })
        );
    }

    /// Bought tickets return 70–90 % of their price on average (game design)
    #[test]
    fn shop_tickets_pay_back_most_of_their_price() {
        let mut rng = StdRng::seed_from_u64(11);
        for s in Series::ALL {
            let Some(price) = s.price() else { continue };
            let ratio = measured_value(s, 60_000, &mut rng) / price as f64;
            assert!((0.68..=0.92).contains(&ratio), "{} pays back {ratio:.2} of its price", s.key());
        }
    }

    #[test]
    fn weighted_follows_the_weights() {
        let mut rng = StdRng::seed_from_u64(3);
        let table = [("a", 1), ("b", 3)];
        let b = (0..40_000).filter(|_| *weighted(&mut rng, &table) == "b").count();
        assert!((b as f64 / 40_000.0 - 0.75).abs() < 0.01);
    }
}
