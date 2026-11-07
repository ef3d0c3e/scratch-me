//! Charms: bought in the shop, they boost play for a while. Rules only, no I/O:
//! routes keep the charms worn and spend their charges.
//!
//! A charm has a kind (what it does) and a tier (how much, on which tickets,
//! for how long). It lasts for a number of charges or a time, whichever runs
//! out first. A player wears at most one charm of each kind; kinds stack.

use chrono::Duration;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::tickets::{Drawn, Prize, Series};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// Tickets bought cost less. A charge per ticket bought.
    Discount,
    /// A losing ticket may get a second draw. A charge per losing ticket.
    Luck,
    /// The coins a ticket pays get a bonus. A charge per ticket finished.
    Payout,
    /// More experience for each ticket scratched. A charge per ticket.
    Scholar,
    /// The pocket refills twice as fast while it lasts (no charges)
    Hourglass,
    /// Quest progress counts double while it lasts (no charges)
    Quest,
    /// Keeps the envelope streak through missed days: a charge per day
    Shield,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Bronze,
    Silver,
    Gold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Charm {
    pub kind: Kind,
    pub tier: Tier,
}

impl Kind {
    pub const ALL: [Kind; 7] =
        [Kind::Discount, Kind::Luck, Kind::Payout, Kind::Scholar, Kind::Hourglass, Kind::Quest, Kind::Shield];

    pub fn key(self) -> &'static str {
        match self {
            Kind::Discount => "discount",
            Kind::Luck => "luck",
            Kind::Payout => "payout",
            Kind::Scholar => "scholar",
            Kind::Hourglass => "hourglass",
            Kind::Quest => "quest",
            Kind::Shield => "shield",
        }
    }

    /// Acts on the coins of tickets bought or scratched: the payback rule
    /// below holds them to it
    pub fn on_coins(self) -> bool {
        matches!(self, Kind::Discount | Kind::Luck | Kind::Payout)
    }

    pub fn parse(key: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.key() == key)
    }
}

impl Tier {
    pub const ALL: [Tier; 3] = [Tier::Bronze, Tier::Silver, Tier::Gold];

    pub fn key(self) -> &'static str {
        match self {
            Tier::Bronze => "bronze",
            Tier::Silver => "silver",
            Tier::Gold => "gold",
        }
    }

    pub fn parse(key: &str) -> Option<Tier> {
        Tier::ALL.into_iter().find(|t| t.key() == key)
    }

    /// 0, 1, 2: for tables by tier
    fn at(self) -> usize {
        self as usize
    }

    /// The dearest ticket a coin charm works on, in coins; None for any
    pub fn reach(self) -> Option<i64> {
        match self {
            Tier::Bronze => Some(60),
            Tier::Silver => Some(150),
            Tier::Gold => None,
        }
    }

    /// The level it takes to buy it
    pub fn level(self) -> i32 {
        match self {
            Tier::Bronze => 2,
            Tier::Silver => 4,
            Tier::Gold => 6,
        }
    }
}

impl Charm {
    pub fn all() -> Vec<Charm> {
        Kind::ALL.into_iter().flat_map(|kind| Tier::ALL.into_iter().map(move |tier| Charm { kind, tier })).collect()
    }

    pub fn parse(kind: &str, tier: &str) -> Option<Charm> {
        Some(Charm { kind: Kind::parse(kind)?, tier: Tier::parse(tier)? })
    }

    /// Uses before it is spent: tickets, or missed days for a shield. The
    /// charms that only last a time have one, never spent.
    pub fn charges(self) -> i32 {
        let t = self.tier.at();
        match self.kind {
            Kind::Discount | Kind::Luck | Kind::Payout => [5, 8, 12][t],
            Kind::Scholar => [10, 20, 40][t],
            Kind::Hourglass | Kind::Quest => 1,
            Kind::Shield => [1, 2, 3][t],
        }
    }

    /// How long it lasts once bought
    pub fn lasts(self) -> Duration {
        let t = self.tier.at();
        match self.kind {
            Kind::Scholar => [Duration::hours(1), Duration::hours(2), Duration::hours(4)][t],
            Kind::Shield => Duration::days([7, 14, 30][t]),
            _ => [Duration::minutes(30), Duration::hours(1), Duration::hours(2)][t],
        }
    }

    /// The dearest ticket it works on (coin charms); None for any
    pub fn reach(self) -> Option<i64> {
        if self.kind.on_coins() { self.tier.reach() } else { None }
    }

    /// Percent: off the price, chance of a second draw, coins or experience
    /// added; how much faster for the hourglass and quest charms
    pub fn strength(self) -> i64 {
        let t = self.tier.at();
        match self.kind {
            Kind::Discount => [15, 20, 25][t],
            Kind::Luck | Kind::Payout => [20, 25, 30][t],
            Kind::Scholar => [25, 50, 100][t],
            Kind::Hourglass | Kind::Quest => 100,
            Kind::Shield => 0,
        }
    }

    /// Shop price for a player at `level`. Coin charms are set by the payback
    /// rule tested below. The hourglass gives pocket tickets, which are worth
    /// more as a player levels: it costs a little less than they are worth.
    pub fn price(self, level: i32) -> i64 {
        let t = self.tier.at();
        match self.kind {
            Kind::Discount => [40, 210, 1_350][t],
            Kind::Luck | Kind::Payout => [50, 260, 1_600][t],
            Kind::Scholar => [60, 150, 400][t],
            Kind::Quest => [100, 250, 500][t],
            Kind::Shield => [300, 700, 1_500][t],
            Kind::Hourglass => {
                // Extra tickets: the minutes it lasts over the refill's, at 95 % of their worth
                let (minutes, refill) = (self.lasts().num_minutes(), crate::pocket::REFILL.num_minutes());
                (minutes * crate::pocket::worth(level) * 95 + refill * 100 - 1) / (refill * 100)
            }
        }
    }

    /// Works on this series: a shop ticket within the tier's reach. Luck also
    /// needs a ticket drawn whole up front, not played move by move or dug.
    /// Experience counts on every ticket; the others are not about tickets.
    pub fn works_on(self, series: Series) -> bool {
        match self.kind {
            Kind::Scholar => return true,
            Kind::Hourglass | Kind::Quest | Kind::Shield => return false,
            _ => {}
        }
        let Some(price) = series.price() else { return false };
        if self.tier.reach().is_some_and(|max| price > max) {
            return false;
        }
        match self.kind {
            Kind::Luck => !series.is_played() && !series.is_dig(),
            _ => true,
        }
    }

    /// Scholar: the extra experience on top of what a ticket gave
    pub fn more_xp(self, xp: i64) -> i64 {
        if self.kind == Kind::Scholar { xp * self.strength() / 100 } else { 0 }
    }

    /// The price of a ticket with this discount
    pub fn discounted(self, price: i64) -> i64 {
        price - self.saved(price)
    }

    /// Coins a discount takes off a price, rounded down
    pub fn saved(self, price: i64) -> i64 {
        price * self.strength() / 100
    }

    /// Coins a payout charm adds to a win, rounded up so a small win still gets one
    pub fn bonus(self, coins: i64) -> i64 {
        if coins <= 0 { 0 } else { (coins * self.strength() + 99) / 100 }
    }

    /// Luck: a losing ticket's second draw, when the charm's chance comes up
    pub fn second_chance(self, series: Series, rng: &mut impl Rng) -> Option<Drawn> {
        (self.kind == Kind::Luck && rng.random_range(0..100) < self.strength() as u32).then(|| series.draw(rng))
    }
}

/// Whether a prize counts as a loss, for luck: nothing at all
pub fn loses(prize: &Prize) -> bool {
    *prize == Prize::Nothing
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    /// The best payback a shop ticket may have (the shop's own test)
    const PAYBACK: f64 = 0.92;

    /// The dearest shop ticket a tier reaches
    fn worst_price(tier: Tier) -> f64 {
        let dearest =
            Series::ALL.into_iter().filter_map(Series::price).filter(|p| tier.reach().is_none_or(|m| *p <= m));
        dearest.max().expect("a ticket") as f64
    }

    /// Return over spending when every charge goes to the dearest ticket the
    /// charms reach, at the best payback. Worst case: luck finds a loser each time,
    /// and the fewest tickets are bought (one per charge of the charm with most).
    fn payback(charms: &[Charm]) -> f64 {
        let p = charms.iter().map(|c| worst_price(c.tier)).fold(f64::MAX, f64::min);
        let tickets = charms.iter().map(|c| c.charges()).max().unwrap_or(0) as f64;
        let value = PAYBACK * p;
        let mut back = tickets * value;
        let mut spent = tickets * p + charms.iter().map(|c| c.price(1) as f64).sum::<f64>();
        for c in charms {
            let n = c.charges() as f64;
            let s = c.strength() as f64 / 100.0;
            match c.kind {
                Kind::Discount => spent -= n * p * s,
                // Luck and payout (only coin charms come here)
                _ => back += n * value * s,
            }
        }
        back / spent
    }

    /// A charm makes heavy play cheaper, never profitable (game design)
    #[test]
    fn charms_never_make_play_pay() {
        for c in Charm::all().into_iter().filter(|c| c.kind.on_coins()) {
            let r = payback(&[c]);
            assert!(r <= 0.95, "{:?} {:?} pays back {r:.3}", c.kind, c.tier);
            // And it is worth having on the tickets it was made for
            assert!(r > PAYBACK, "{:?} {:?} pays back only {r:.3}", c.kind, c.tier);
        }
        for tier in Tier::ALL {
            let all: Vec<Charm> =
                Kind::ALL.into_iter().filter(|k| k.on_coins()).map(|kind| Charm { kind, tier }).collect();
            let r = payback(&all);
            assert!(r <= 0.98, "all {tier:?} charms together pay back {r:.3}");
        }
    }

    /// An hourglass's extra pocket tickets are worth a bit more than it costs
    /// to someone who claims them all, never twice its price, at any level
    #[test]
    fn an_hourglass_costs_about_what_its_tickets_are_worth() {
        for level in [1, 2, 5, 10, 30, 50] {
            for tier in Tier::ALL {
                let c = Charm { kind: Kind::Hourglass, tier };
                let extra = c.lasts().num_minutes() as f64 / crate::pocket::REFILL.num_minutes() as f64;
                let worth = extra * PAYBACK * crate::pocket::worth(level) as f64;
                let r = worth / c.price(level) as f64;
                assert!((0.85..=1.0).contains(&r), "{tier:?} at level {level}: {r:.3}");
            }
        }
    }

    #[test]
    fn the_charms_beyond_coins() {
        let scholar = Charm { kind: Kind::Scholar, tier: Tier::Gold };
        assert_eq!(scholar.more_xp(30), 30, "double");
        assert!(scholar.works_on(Series::Rocket) && scholar.works_on(Series::Golden), "every ticket");
        assert_eq!(Charm { kind: Kind::Payout, tier: Tier::Gold }.more_xp(30), 0);
        let shield = Charm { kind: Kind::Shield, tier: Tier::Silver };
        assert_eq!((shield.charges(), shield.lasts()), (2, Duration::days(14)));
        assert!(
            !shield.works_on(Series::LuckyThree)
                && !Charm { kind: Kind::Quest, tier: Tier::Bronze }.works_on(Series::LuckyThree)
        );
        assert_eq!(Charm { kind: Kind::Hourglass, tier: Tier::Gold }.charges(), 1, "time only");
    }

    #[test]
    fn dearer_tiers_do_more() {
        for kind in Kind::ALL {
            let [b, s, g] = Tier::ALL.map(|tier| Charm { kind, tier });
            assert!(b.strength() <= s.strength() && s.strength() <= g.strength());
            assert!(b.price(10) < s.price(10) && s.price(10) < g.price(10));
            assert!(b.charges() <= g.charges() && b.lasts() < g.lasts());
        }
    }

    #[test]
    fn reach() {
        let bronze = Charm { kind: Kind::Payout, tier: Tier::Bronze };
        let gold = Charm { kind: Kind::Payout, tier: Tier::Gold };
        assert!(bronze.works_on(Series::LuckyThree));
        assert!(!bronze.works_on(Series::CashWheel));
        assert!(gold.works_on(Series::CashWheel));
        assert!(gold.works_on(Series::Rocket), "payout pays played games too");
        assert!(!gold.works_on(Series::Duel), "not a shop ticket");
        assert!(!gold.works_on(Series::Golden));
        let luck = Charm { kind: Kind::Luck, tier: Tier::Gold };
        assert!(luck.works_on(Series::CashWheel));
        assert!(!luck.works_on(Series::Rocket), "played move by move");
        assert!(!luck.works_on(Series::TreasureDig));
    }

    #[test]
    fn sums() {
        let d = Charm { kind: Kind::Discount, tier: Tier::Silver };
        assert_eq!(d.discounted(100), 80);
        assert_eq!(d.discounted(35), 28);
        let p = Charm { kind: Kind::Payout, tier: Tier::Bronze };
        assert_eq!(p.bonus(100), 20);
        assert_eq!(p.bonus(1), 1);
        assert_eq!(p.bonus(0), 0);
    }

    #[test]
    fn luck_comes_up_at_its_chance() {
        let mut rng = StdRng::seed_from_u64(5);
        let c = Charm { kind: Kind::Luck, tier: Tier::Silver };
        let hits = (0..20_000).filter(|_| c.second_chance(Series::LuckyThree, &mut rng).is_some()).count();
        assert!((hits as f64 / 20_000.0 - 0.25).abs() < 0.015);
        let p = Charm { kind: Kind::Payout, tier: Tier::Gold };
        assert!((0..100).all(|_| p.second_chance(Series::LuckyThree, &mut rng).is_none()));
    }

    #[test]
    fn keys_round_trip() {
        for c in Charm::all() {
            assert_eq!(Charm::parse(c.kind.key(), c.tier.key()), Some(c));
        }
        assert_eq!(Charm::parse("luck", "platinum"), None);
    }
}
