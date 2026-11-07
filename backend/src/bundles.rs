//! Bundles: several tickets bought at once, possibly from different tiers,
//! each carrying the bundle's seal. Not tiered: they run from a cheap
//! starter to a vault of top-tier tickets. Rules only, no I/O: the shop
//! issues the tickets.

use serde::Serialize;

use crate::progress::levels;
use crate::tickets::Series;
use crate::tickets::seals::{self, Seal};

#[derive(Debug, Clone, Serialize)]
pub struct Bundle {
    pub key: &'static str,
    pub price: i64,
    #[serde(serialize_with = "seal_key")]
    pub seal: Seal,
    /// In the order they land in the stack
    pub tickets: &'static [Series],
}

fn seal_key<S: serde::Serializer>(seal: &Seal, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(seal.key)
}

fn seal(key: &str) -> Seal {
    seals::seal(key).expect("a seal")
}

/// Every bundle, cheapest first. Prices are set by the payback test below.
pub fn all() -> Vec<Bundle> {
    use Series::*;
    vec![
        Bundle {
            key: "starter",
            price: 200,
            seal: seal("sunrise"),
            tickets: &[LuckyThree, TicTacToe, BoardWalk, LuckyDice],
        },
        Bundle {
            key: "encore",
            price: 480,
            seal: seal("echo"),
            tickets: &[LuckyReels, WildReels, HighCard, LuckyDice, Blackjack],
        },
        Bundle { key: "safety", price: 540, seal: seal("safety"), tickets: &[Plinko, Derby, Bingo, NumberMatch] },
        Bundle {
            key: "beach",
            price: 1150,
            seal: seal("beach"),
            tickets: &[Fishing, Fishing, Battleship, TreasureDig],
        },
        Bundle {
            key: "fortune",
            price: 1280,
            seal: seal("fortune"),
            tickets: &[Bingo, NumberMatch, HotSpot, PokerHand, WildReels],
        },
        Bundle {
            key: "casino",
            price: 1700,
            seal: seal("casino"),
            tickets: &[CoinFlip, Roulette, Blackjack, PokerHand, MegaReels],
        },
        Bundle {
            key: "diamond",
            price: 4600,
            seal: seal("diamond"),
            tickets: &[LuckyThree, LuckyThree, LuckyThree, LuckyThree, LuckyThree],
        },
        Bundle {
            key: "cosmos",
            price: 6200,
            seal: seal("cosmos"),
            tickets: &[PlatinumThree, PlatinumThree, CashWheel, CashWheel, SafeCracker],
        },
        // In season only: while every ticket in it is
        Bundle {
            key: "halloween",
            price: 650,
            seal: seal("halloween"),
            tickets: &[PumpkinPatch, PumpkinPatch, PumpkinPatch],
        },
        Bundle { key: "winter", price: 520, seal: seal("winter"), tickets: &[Snowball, Snowball, Snowball] },
    ]
}

pub fn bundle(key: &str) -> Option<Bundle> {
    all().into_iter().find(|b| b.key == key)
}

impl Bundle {
    /// The level it takes: that of its highest tier
    pub fn level(&self) -> i32 {
        self.tickets.iter().map(|s| levels::shop_level(*s)).max().unwrap_or(1)
    }

    /// On sale this day: every ticket in it is in season
    pub fn in_season(&self, day: chrono::NaiveDate) -> bool {
        self.tickets.iter().all(|s| s.in_season(day))
    }

    /// Sold all year
    pub fn seasonal(&self) -> bool {
        self.tickets.iter().any(|s| s.season().is_some())
    }

    /// What its tickets cost one by one
    pub fn separately(&self) -> i64 {
        self.tickets.iter().filter_map(|s| s.price()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tickets::sealed_value;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn bundles_hold_shop_tickets_their_seal_fits() {
        let all = all();
        let keys: std::collections::HashSet<_> = all.iter().map(|b| b.key).collect();
        assert_eq!(keys.len(), all.len());
        for b in &all {
            assert!(b.tickets.len() >= 3, "{}", b.key);
            for s in b.tickets {
                assert!(s.in_shop(), "{} holds {}, not a shop ticket", b.key, s.key());
                assert!(b.seal.fits(*s), "{}: its seal cannot go on {}", b.key, s.key());
            }
        }
        let year: Vec<&Bundle> = all.iter().filter(|b| !b.seasonal()).collect();
        assert!(year.windows(2).all(|w| w[0].price < w[1].price), "cheapest first");
        assert!(year.last().unwrap().price >= 20 * year[0].price, "a wide range of prices");
        let d = |s: &str| s.parse::<chrono::NaiveDate>().unwrap();
        let halloween = bundle("halloween").unwrap();
        assert!(halloween.in_season(d("2026-10-31")) && !halloween.in_season(d("2026-12-25")));
        assert!(bundle("winter").unwrap().in_season(d("2026-12-25")));
    }

    /// A bundle pays back better than its tickets one by one, never more than its price
    #[test]
    fn bundles_pay_back_most_of_their_price() {
        let mut rng = StdRng::seed_from_u64(12);
        let off: Vec<String> = all()
            .into_iter()
            .filter_map(|b| {
                let value: f64 = b.tickets.iter().map(|s| sealed_value(*s, b.seal, 40_000, &mut rng)).sum();
                let ratio = value / b.price as f64;
                (!(0.82..=0.95).contains(&ratio))
                    .then(|| format!("{} worth {value:.0} for {}: {ratio:.3}", b.key, b.price))
            })
            .collect();
        assert!(off.is_empty(), "bundles off their payback: {off:?}");
    }
}
