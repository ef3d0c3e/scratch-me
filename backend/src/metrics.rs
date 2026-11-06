//! Prometheus metrics, as in AnyCardz. Prometheus runs elsewhere and scrapes
//! `GET /metrics`, served on its own listener (SCRATCHME_METRICS_BIND), never
//! through the public site. A bearer token (SCRATCHME_METRICS_TOKEN) can guard
//! it too.
//!
//! Counters only grow while the process runs; Grafana reads them through
//! `rate()`/`increase()`, which handle restarts. Gauges (sockets, pool,
//! players) are read at scrape time. Handlers count after their transaction
//! commits: what was rolled back never happened.
//!
//! Every label value comes from a closed set (series, sources, quest keys,
//! achievement ids, route patterns): no series grows with players or paths.

use std::time::Instant;

use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use actix_web::{HttpRequest, HttpResponse, get, web};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::{Histogram, exponential_buckets};
use prometheus_client::registry::Registry;

use crate::AppState;
use crate::error::{ApiError, ApiResult};
use crate::progress::Progress;
use crate::routes::tickets::TicketDto;
use crate::tickets::Prize;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct HttpLabels {
    method: String,
    /// The route pattern (`/api/tickets/{id}/play`), never the raw path: bounded
    route: String,
    status: u16,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct RouteLabels {
    method: String,
    route: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct IssuedLabels {
    series: String,
    /// pocket, envelope, prize, shop, reward, gift, duel
    source: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct FinishedLabels {
    series: String,
    /// win, lose
    result: &'static str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct SeriesLabels {
    series: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ReasonLabels {
    reason: &'static str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct KeyLabels {
    key: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct KindLabels {
    kind: &'static str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct StatusLabels {
    status: &'static str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct AuthLabels {
    /// login, register, guest, claim
    action: &'static str,
    /// ok, failed, limited
    result: &'static str,
}

pub struct Metrics {
    registry: Registry,
    http_requests: Family<HttpLabels, Counter>,
    http_duration: Family<RouteLabels, Histogram>,
    auth: Family<AuthLabels, Counter>,
    issued: Family<IssuedLabels, Counter>,
    finished: Family<FinishedLabels, Counter>,
    prize_coins: Family<SeriesLabels, Counter>,
    coins_earned: Family<ReasonLabels, Counter>,
    coins_spent: Family<ReasonLabels, Counter>,
    pocket_claims: Counter,
    envelope_opens: Counter,
    quests: Family<KeyLabels, Counter>,
    achievements: Family<KeyLabels, Counter>,
    level_ups: Counter,
    cosmetics: Family<KindLabels, Counter>,
    shouts: Counter,
    gifts: Counter,
    duels: Family<StatusLabels, Counter>,
    pot_entries: Counter,
    pots_won: Counter,
    guests_forgotten: Counter,
    sockets: Gauge,
    online: Gauge,
    players: Family<KindLabels, Gauge>,
    pool_size: Gauge,
    pool_available: Gauge,
    pool_waiting: Gauge,
}

impl Default for Metrics {
    fn default() -> Self {
        let mut r = Registry::with_prefix("scratchme");
        let http_requests = Family::<HttpLabels, Counter>::default();
        // 5 ms to ~10 s
        let http_duration = Family::<RouteLabels, Histogram>::new_with_constructor(|| {
            Histogram::new(exponential_buckets(0.005, 2.0, 12))
        });
        let m = Metrics {
            http_requests,
            http_duration,
            auth: Default::default(),
            issued: Default::default(),
            finished: Default::default(),
            prize_coins: Default::default(),
            coins_earned: Default::default(),
            coins_spent: Default::default(),
            pocket_claims: Default::default(),
            envelope_opens: Default::default(),
            quests: Default::default(),
            achievements: Default::default(),
            level_ups: Default::default(),
            cosmetics: Default::default(),
            shouts: Default::default(),
            gifts: Default::default(),
            duels: Default::default(),
            pot_entries: Default::default(),
            pots_won: Default::default(),
            guests_forgotten: Default::default(),
            sockets: Default::default(),
            online: Default::default(),
            players: Default::default(),
            pool_size: Default::default(),
            pool_available: Default::default(),
            pool_waiting: Default::default(),
            registry: Registry::default(),
        };
        r.register("http_requests", "HTTP requests answered", m.http_requests.clone());
        r.register("http_request_duration_seconds", "Time to answer HTTP requests", m.http_duration.clone());
        r.register("auth_attempts", "Logins, sign-ups, guests and guests signing up, by result", m.auth.clone());
        r.register("tickets_issued", "Tickets given to players, by series and source", m.issued.clone());
        r.register("tickets_finished", "Tickets scratched or played to the end, by result", m.finished.clone());
        r.register("ticket_prize_coins", "Coins paid by tickets, by series (payback)", m.prize_coins.clone());
        r.register("coins_earned", "Coins entering players' balances, by reason", m.coins_earned.clone());
        r.register("coins_spent", "Coins leaving players' balances, by reason", m.coins_spent.clone());
        r.register("pocket_claims", "Pockets emptied into the stack", m.pocket_claims.clone());
        r.register("envelope_opens", "Daily envelopes opened", m.envelope_opens.clone());
        r.register("quests_done", "Daily quests finished, by quest (bonus: all three)", m.quests.clone());
        r.register("achievements", "Achievements earned, by achievement", m.achievements.clone());
        r.register("level_ups", "Levels reached", m.level_ups.clone());
        r.register("cosmetics_bought", "Cosmetics bought, by kind", m.cosmetics.clone());
        r.register("shouts", "Shoutbox messages posted", m.shouts.clone());
        r.register("gifts", "Tickets sent to another player", m.gifts.clone());
        r.register("duels", "Duels by what happened (open, accepted, declined, cancelled, expired)", m.duels.clone());
        r.register("pot_entries", "Entries in the daily pot", m.pot_entries.clone());
        r.register("pots_won", "Daily pots drawn with a winner", m.pots_won.clone());
        r.register("guests_forgotten", "Guest accounts deleted, no session left", m.guests_forgotten.clone());
        r.register("live_sockets", "Open live sockets", m.sockets.clone());
        r.register("online_users", "Players with a live socket open", m.online.clone());
        r.register("players", "Accounts, by kind (member, guest)", m.players.clone());
        r.register("db_pool_connections", "Database connections in the pool", m.pool_size.clone());
        r.register("db_pool_available", "Idle database connections", m.pool_available.clone());
        r.register("db_pool_waiting", "Requests waiting for a database connection", m.pool_waiting.clone());
        Metrics { registry: r, ..m }
    }
}

/// How an auth attempt went, from its answer
pub fn auth_result<T>(r: &ApiResult<T>) -> &'static str {
    match r {
        Ok(_) => "ok",
        Err(ApiError::RetryAfter(_)) => "limited",
        Err(_) => "failed",
    }
}

impl Metrics {
    /// `action`: login, register, guest, claim
    pub fn auth(&self, action: &'static str, result: &'static str) {
        self.auth.get_or_create(&AuthLabels { action, result }).inc();
    }

    /// Tickets given, whatever gave them
    pub fn issued(&self, tickets: &[TicketDto]) {
        for t in tickets {
            self.issued.get_or_create(&IssuedLabels { series: t.series.clone(), source: t.source.clone() }).inc();
        }
    }

    /// A ticket scratched or played out, and what it paid (`extra`: a duplicate cosmetic's coins)
    pub fn finished(&self, series: &str, prize: &Prize, extra: i64) {
        let coins = prize.coins() + extra;
        let result = if *prize == Prize::Nothing { "lose" } else { "win" };
        self.finished.get_or_create(&FinishedLabels { series: series.to_owned(), result }).inc();
        self.prize_coins.get_or_create(&SeriesLabels { series: series.to_owned() }).inc_by(coins.max(0) as u64);
        self.earned("prize", coins);
    }

    /// `reason`: start, prize, reward, envelope, pot, refund
    pub fn earned(&self, reason: &'static str, coins: i64) {
        self.coins_earned.get_or_create(&ReasonLabels { reason }).inc_by(coins.max(0) as u64);
    }

    /// `reason`: ticket, gift, cosmetic, duel_stake
    pub fn spent(&self, reason: &'static str, coins: i64) {
        self.coins_spent.get_or_create(&ReasonLabels { reason }).inc_by(coins.max(0) as u64);
    }

    /// What an action's progress gave: quests, achievements, levels, their rewards
    pub fn progress(&self, p: &Progress) {
        let mut coins = 0;
        for q in &p.quests {
            self.quests.get_or_create(&KeyLabels { key: q.what.clone() }).inc();
            coins += q.reward.coins();
        }
        if let Some(bonus) = &p.bonus {
            self.quests.get_or_create(&KeyLabels { key: "bonus".into() }).inc();
            coins += bonus.coins();
        }
        for a in &p.achievements {
            self.achievements.get_or_create(&KeyLabels { key: a.what.clone() }).inc();
            coins += a.reward.coins();
        }
        for l in &p.level_ups {
            self.level_ups.inc();
            coins += l.reward.coins();
        }
        self.earned("reward", coins);
        self.issued(&p.tickets);
    }

    pub fn pocket_claim(&self) {
        self.pocket_claims.inc();
    }

    pub fn envelope_open(&self, coins: i64) {
        self.envelope_opens.inc();
        self.earned("envelope", coins);
    }

    /// `kind`: theme, tool, foil
    pub fn cosmetic(&self, kind: &'static str, price: i64) {
        self.cosmetics.get_or_create(&KindLabels { kind }).inc();
        self.spent("cosmetic", price);
    }

    pub fn shout(&self) {
        self.shouts.inc();
    }

    pub fn gift(&self) {
        self.gifts.inc();
    }

    /// `status`: open, accepted, declined, cancelled, expired
    pub fn duel(&self, status: &'static str) {
        self.duels.get_or_create(&StatusLabels { status }).inc();
    }

    pub fn pot_entry(&self) {
        self.pot_entries.inc();
    }

    pub fn pot_won(&self, coins: i64) {
        self.pots_won.inc();
        self.earned("pot", coins);
    }

    pub fn guests_forgotten(&self, n: usize) {
        self.guests_forgotten.inc_by(n as u64);
    }

    fn http(&self, method: &str, route: &str, status: u16, seconds: Option<f64>) {
        let (method, route) = (method.to_owned(), route.to_owned());
        if let Some(s) = seconds {
            self.http_duration.get_or_create(&RouteLabels { method: method.clone(), route: route.clone() }).observe(s);
        }
        self.http_requests.get_or_create(&HttpLabels { method, route, status }).inc();
    }

    /// The text exposition, with the gauges read now
    pub async fn render(&self, state: &AppState) -> String {
        let (sockets, online) = state.live.counts();
        self.sockets.set(sockets as i64);
        self.online.set(online as i64);
        let pool = state.pool.status();
        self.pool_size.set(pool.size as i64);
        self.pool_available.set(pool.available as i64);
        self.pool_waiting.set(pool.waiting as i64);
        if let Ok(mut conn) = state.pool.get().await {
            use crate::schema::users;
            let kinds: Result<Vec<(bool, i64)>, _> = users::table
                .group_by(users::is_guest)
                .select((users::is_guest, diesel::dsl::count_star()))
                .load(&mut conn)
                .await;
            match kinds {
                Ok(rows) => {
                    // Both always shown, 0 when there are none
                    for (guest, kind) in [(false, "member"), (true, "guest")] {
                        let n = rows.iter().find(|(g, _)| *g == guest).map_or(0, |(_, n)| *n);
                        self.players.get_or_create(&KindLabels { kind }).set(n);
                    }
                }
                Err(e) => tracing::warn!("metrics: counting players: {e}"),
            }
        }
        let mut out = String::new();
        prometheus_client::encoding::text::encode(&mut out, &self.registry).expect("writing to a String");
        out
    }
}

/// Counts and times every request by route pattern. Wrap the App with
/// `actix_web::middleware::from_fn(metrics::track)`.
pub async fn track(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse, actix_web::Error> {
    let start = Instant::now();
    let state = req.app_data::<web::Data<AppState>>().cloned();
    let method = req.method().to_string();
    let res = next.call(req).await?;
    if let Some(state) = state {
        // Unknown paths share one label: scanners cannot grow the series
        let route = res.request().match_pattern().unwrap_or_else(|| "unmatched".to_owned());
        // A live socket stays open for hours: its time says nothing about latency
        let seconds = (route != "/api/live").then(|| start.elapsed().as_secs_f64());
        state.metrics.http(&method, &route, res.status().as_u16(), seconds);
    }
    Ok(res.map_into_boxed_body())
}

/// Compares without stopping at the first difference
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[get("/metrics")]
pub async fn scrape(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    if let Some(token) = &state.config.metrics_token {
        let given =
            req.headers().get("authorization").and_then(|h| h.to_str().ok()).and_then(|h| h.strip_prefix("Bearer "));
        if !given.is_some_and(|g| constant_time_eq(g.as_bytes(), token.as_bytes())) {
            return HttpResponse::Unauthorized().insert_header(("www-authenticate", "Bearer")).finish();
        }
    }
    HttpResponse::Ok()
        .content_type("application/openmetrics-text; version=1.0.0; charset=utf-8")
        .body(state.metrics.render(&state).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::Reward;
    use crate::progress::levels;

    fn text(m: &Metrics) -> String {
        let mut out = String::new();
        prometheus_client::encoding::text::encode(&mut out, &m.registry).unwrap();
        out
    }

    #[test]
    fn counters_show_with_their_labels() {
        let m = Metrics::default();
        m.finished("lucky_three", &Prize::Coins { amount: 50 }, 0);
        m.finished("lucky_three", &Prize::Nothing, 0);
        m.spent("ticket", 35);
        m.auth("guest", "limited");
        m.duel("accepted");
        m.http("POST", "/api/tickets/{id}/scratch", 200, Some(0.01));
        let t = text(&m);
        assert!(t.contains(r#"scratchme_tickets_finished_total{series="lucky_three",result="win"} 1"#), "{t}");
        assert!(t.contains(r#"scratchme_tickets_finished_total{series="lucky_three",result="lose"} 1"#), "{t}");
        assert!(t.contains(r#"scratchme_ticket_prize_coins_total{series="lucky_three"} 50"#), "{t}");
        assert!(t.contains(r#"scratchme_coins_earned_total{reason="prize"} 50"#), "{t}");
        assert!(t.contains(r#"scratchme_coins_spent_total{reason="ticket"} 35"#), "{t}");
        assert!(t.contains(r#"scratchme_auth_attempts_total{action="guest",result="limited"} 1"#), "{t}");
        assert!(t.contains(r#"scratchme_duels_total{status="accepted"} 1"#), "{t}");
        assert!(t.contains(
            r#"scratchme_http_requests_total{method="POST",route="/api/tickets/{id}/scratch",status="200"} 1"#
        ));
        assert!(t.contains(
            r#"scratchme_http_request_duration_seconds_count{method="POST",route="/api/tickets/{id}/scratch"} 1"#
        ));
    }

    #[test]
    fn progress_counts_its_rewards() {
        let m = Metrics::default();
        let coins = |amount| Prize::Coins { amount };
        let p = Progress {
            xp_gained: 10,
            level: levels::dto(0),
            level_ups: vec![Reward { what: "2".into(), reward: coins(0) }],
            quests: vec![Reward { what: "scratch_5".into(), reward: coins(100) }],
            bonus: Some(coins(25)),
            achievements: vec![Reward { what: "first_win".into(), reward: coins(50) }],
            tickets: vec![],
            coins: 0,
        };
        m.progress(&p);
        let t = text(&m);
        assert!(t.contains(r#"scratchme_quests_done_total{key="scratch_5"} 1"#), "{t}");
        assert!(t.contains(r#"scratchme_quests_done_total{key="bonus"} 1"#), "{t}");
        assert!(t.contains(r#"scratchme_achievements_total{key="first_win"} 1"#), "{t}");
        assert!(t.contains("scratchme_level_ups_total 1"), "{t}");
        assert!(t.contains(r#"scratchme_coins_earned_total{reason="reward"} 175"#), "{t}");
    }

    #[test]
    fn negative_amounts_never_lower_a_counter() {
        let m = Metrics::default();
        m.earned("refund", -5);
        assert!(text(&m).contains(r#"scratchme_coins_earned_total{reason="refund"} 0"#));
    }

    #[test]
    fn auth_results() {
        assert_eq!(auth_result::<()>(&Ok(())), "ok");
        assert_eq!(auth_result::<()>(&Err(ApiError::RetryAfter(std::time::Duration::from_secs(1)))), "limited");
        assert_eq!(auth_result::<()>(&Err(ApiError::Invalid("weak_password"))), "failed");
    }
}
