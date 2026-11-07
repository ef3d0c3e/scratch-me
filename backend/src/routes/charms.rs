//! Charms worn, and spending their charges. Bought through the shop
//! (`shop.rs`); spent when buying (discount) and scratching (luck, payout).
//! Every charge is spent in the caller's transaction.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::Serialize;

use crate::charms::{Charm, Kind, Tier};
use crate::error::{ApiError, ApiResult};
use crate::schema::{charms, tickets, users};
use crate::tickets::{Prize, Series};

/// A charm the player wears now
#[derive(Debug, Clone, Serialize)]
pub struct WornCharm {
    pub kind: Kind,
    pub tier: Tier,
    /// Percent: off, chance, or added
    pub strength: i64,
    /// The dearest ticket it works on; None for any
    pub reach: Option<i64>,
    pub charges: i32,
    pub expires_at: DateTime<Utc>,
}

/// Charms worn now: charges left and time to go, in kind order
pub(crate) async fn worn(conn: &mut AsyncPgConnection, uid: i64) -> ApiResult<Vec<WornCharm>> {
    let rows: Vec<(String, String, i32, DateTime<Utc>)> = charms::table
        .filter(charms::user_id.eq(uid))
        .filter(charms::charges.gt(0))
        .filter(charms::expires_at.gt(Utc::now()))
        .select((charms::kind, charms::tier, charms::charges, charms::expires_at))
        .load(conn)
        .await?;
    let mut out: Vec<WornCharm> = rows
        .into_iter()
        .filter_map(|(k, t, charges, expires_at)| {
            let c = Charm::parse(&k, &t)?;
            Some(WornCharm {
                kind: c.kind,
                tier: c.tier,
                strength: c.strength(),
                reach: c.tier.reach(),
                charges,
                expires_at,
            })
        })
        .collect();
    out.sort_by_key(|c| Kind::ALL.iter().position(|k| *k == c.kind));
    Ok(out)
}

/// Puts a charm on, unless one of its kind is still going
pub(crate) async fn wear(conn: &mut AsyncPgConnection, uid: i64, charm: Charm) -> ApiResult<()> {
    let now = Utc::now();
    // A spent or expired charm of the kind makes room; a live one stays and refuses
    diesel::delete(
        charms::table
            .filter(charms::user_id.eq(uid))
            .filter(charms::kind.eq(charm.kind.key()))
            .filter(charms::charges.eq(0).or(charms::expires_at.le(now))),
    )
    .execute(conn)
    .await?;
    let n = diesel::insert_into(charms::table)
        .values((
            charms::user_id.eq(uid),
            charms::kind.eq(charm.kind.key()),
            charms::tier.eq(charm.tier.key()),
            charms::charges.eq(charm.tier.charges()),
            charms::bought_at.eq(now),
            charms::expires_at.eq(now + charm.tier.lasts()),
        ))
        .on_conflict_do_nothing()
        .execute(conn)
        .await?;
    if n == 0 {
        return Err(ApiError::Conflict("charm_active"));
    }
    Ok(())
}

/// Spends a charge of the player's charm of `kind`, if one is worn and works on `series`
async fn take(conn: &mut AsyncPgConnection, uid: i64, kind: Kind, series: Series) -> ApiResult<Option<Charm>> {
    let tiers: Vec<&str> =
        Tier::ALL.into_iter().filter(|&tier| Charm { kind, tier }.works_on(series)).map(Tier::key).collect();
    if tiers.is_empty() {
        return Ok(None);
    }
    // One statement: two requests at once cannot spend the last charge twice
    let tier: Option<String> = diesel::update(
        charms::table
            .filter(charms::user_id.eq(uid))
            .filter(charms::kind.eq(kind.key()))
            .filter(charms::charges.gt(0))
            .filter(charms::expires_at.gt(Utc::now()))
            .filter(charms::tier.eq_any(tiers)),
    )
    .set(charms::charges.eq(charms::charges - 1))
    .returning(charms::tier)
    .get_result(conn)
    .await
    .optional()?;
    Ok(tier.and_then(|t| Charm::parse(kind.key(), &t)))
}

/// The price of a ticket bought now, and the coins a discount charm took off
pub(crate) async fn discount(
    conn: &mut AsyncPgConnection,
    uid: i64,
    series: Series,
    price: i64,
) -> ApiResult<(i64, i64)> {
    Ok(match take(conn, uid, Kind::Discount, series).await? {
        Some(c) => (c.discounted(price), c.saved(price)),
        None => (price, 0),
    })
}

/// What charms did to a finished ticket
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Boost {
    /// A luck charge was spent
    pub luck: bool,
    /// ...and it drew the ticket again
    pub second_chance: bool,
    /// A payout charge was spent
    pub payout: bool,
    /// Coins it added
    pub bonus: i64,
    /// Coins a seal added (a ticket from a bundle)
    pub seal: i64,
    /// An echo seal gave one more ticket
    pub echo: bool,
}

/// A losing ticket being scratched: a luck charm may draw it again. The new
/// draw replaces the stored one, and is what the player uncovers.
pub(crate) async fn luck(
    conn: &mut AsyncPgConnection,
    uid: i64,
    id: i64,
    series: Series,
    layout: &mut serde_json::Value,
    prize: &mut Prize,
    boost: &mut Boost,
) -> ApiResult<()> {
    if !crate::charms::loses(prize) {
        return Ok(());
    }
    let Some(charm) = take(conn, uid, Kind::Luck, series).await? else { return Ok(()) };
    // The charge is spent either way: the chance was taken
    boost.luck = true;
    let Some(drawn) = charm.second_chance(series, &mut rand::rng()) else { return Ok(()) };
    diesel::update(tickets::table.find(id))
        .set((
            tickets::layout.eq(&drawn.layout),
            tickets::prize.eq(serde_json::to_value(&drawn.prize).expect("prize serializes")),
        ))
        .execute(conn)
        .await?;
    boost.second_chance = true;
    *layout = drawn.layout;
    *prize = drawn.prize;
    Ok(())
}

/// A finished ticket: a payout charm adds to the coins it paid
pub(crate) async fn payout(
    conn: &mut AsyncPgConnection,
    uid: i64,
    series: Series,
    prize: &Prize,
    boost: &mut Boost,
) -> ApiResult<()> {
    let Some(charm) = take(conn, uid, Kind::Payout, series).await? else { return Ok(()) };
    boost.payout = true;
    boost.bonus = charm.bonus(prize.coins());
    if boost.bonus > 0 {
        diesel::update(users::table.find(uid)).set(users::coins.eq(users::coins + boost.bonus)).execute(conn).await?;
    }
    Ok(())
}

impl Boost {
    /// Counted once committed
    pub(crate) fn count(&self, m: &crate::metrics::Metrics) {
        if self.luck {
            m.charm_used(Kind::Luck, 0);
        }
        if self.payout {
            m.charm_used(Kind::Payout, self.bonus);
        }
        if self.seal > 0 {
            m.earned("seal", self.seal);
        }
    }
}
