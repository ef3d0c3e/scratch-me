# Scratch-me

[![CI](https://github.com/ef3d0c3e/scratch-me/actions/workflows/ci.yml/badge.svg)](https://github.com/ef3d0c3e/scratch-me/actions/workflows/ci.yml)

A scratch-ticket simulator for phone browsers, an offshoot of AnyCardz. Free tickets drop on timers, you scratch them with your finger, and they pay out coins, more tickets or cosmetics. No real money, ever.

- `backend/`: API in Rust, actix-web 4 + Diesel (diesel-async, Postgres). It holds every outcome: a ticket's layout and prize are drawn when it is issued and only sent when it is scratched.
- `frontend/`: SvelteKit 3 app (adapter-node, SSR), phone first.

The game design, roadmap and tickets are in Lific, project `SCRAT` (design page SCRAT-DOC-1, roadmap SCRAT-PLAN-1).

## Getting started

You need Rust (edition 2024), Node 24 and Docker.

```sh
cp .env.example .env
(cd frontend && npm install)
make dev          # Postgres in Docker (port 5434), API on :8081, web on http://localhost:5174
```

The ports differ from AnyCardz's, so both can run side by side. The API runs its migrations on startup. New accounts start with 500 coins and a full pocket.

| Command         | What it does                                                         |
| --------------- | -------------------------------------------------------------------- |
| `make dev`      | Database, API and web dev server together                            |
| `make test`     | Backend unit + API integration tests (Postgres), frontend unit tests |
| `make test-e2e` | Browser tests (Playwright) on phone and desktop sizes                |
| `make check`    | clippy, rustfmt, svelte-check, prettier                              |
| `make fmt`      | Format Rust and frontend code                                        |

Announcements are posted by admins, from the Chat page. To make a player an admin:

```sh
cargo run -p scratchme-api -- make-admin <username>
```

## What is in so far

- Accounts (sign up, log in, log out), with the player's time zone
- Lucky Three: a 3×3 grid where three of a kind wins coins, another ticket or a pocket refill
- Pocket: one free ticket every 20 minutes, worth more as you level, holds 3
- Daily envelope: one per calendar day, with a 7-day streak cycle
- Scratch canvas: finger-speed brush, falling foil flakes, the rest falls off once every square is mostly scratched
- Five ticket series (Lucky Three, Number Match, Ticket Chain, Cash Wheel, Golden Ticket), a shop, themes and scratch tools
- Charms: discount, luck (a losing ticket may be drawn again) and payout, in bronze, silver and gold; each lasts a number of uses or a time window, one of each kind at a time, and never makes play profitable
- Celebrations scaled to the prize, scratch sound and haptics, tilt shimmer
- Levels, daily quests, achievements
- Shoutbox and live feed of big wins over a WebSocket, announcements, profiles and titles

## Metrics

Prometheus and Grafana run elsewhere (on the production server, the ones AnyCardz already uses); the API only exposes metrics. They are served on their own listener, `SCRATCHME_METRICS_BIND` (`GET /metrics`, OpenMetrics text), never through the public site, and only when it is set. When `SCRATCHME_METRICS_TOKEN` is set, scrapes must send `Authorization: Bearer <token>`.

Next to AnyCardz (which has 9100), use `SCRATCHME_METRICS_BIND=127.0.0.1:9101`: only the host reaches it.

- `deploy/prometheus/scrape.yml`: the `scratchme` job to add to that Prometheus
- `deploy/grafana/scratchme.json`: the dashboard (Dashboards → Import, pick the Prometheus data source): traffic, latency, sign-ups and guests, tickets and their payout per series, coins in and out (net coin flow shows inflation), drops, progression, social and the DB pool

Check a scrape with `curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:9101/metrics | head`; in Prometheus, Status → Targets shows the `scratchme` job up.

Every metric is prefixed `scratchme_`. Counters are counted once the action's transaction has committed.

| Metric | Labels | What it counts |
| --- | --- | --- |
| `http_requests_total` | method, route, status | Requests answered, by route pattern (`unmatched` for unknown paths) |
| `http_request_duration_seconds` | method, route | Time to answer (the live socket left out) |
| `auth_attempts_total` | action (login, register, guest, claim), result (ok, failed, limited) | Logins, sign-ups, guests, guests signing up |
| `tickets_issued_total` | series, source | Tickets given: pocket, envelope, prize, shop, reward, gift, duel |
| `tickets_finished_total` | series, result (win, lose) | Tickets scratched or played to the end |
| `ticket_prize_coins_total` | series | Coins tickets paid: divided by tickets finished, the payout per ticket |
| `coins_earned_total` | reason (start, prize, reward, envelope, pot, refund, charm) | Coins entering balances |
| `coins_spent_total` | reason (ticket, gift, cosmetic, charm, duel_stake) | Coins leaving balances |
| `pocket_claims_total`, `envelope_opens_total` | | Free drops taken |
| `quests_done_total` | key (quest, or bonus) | Daily quests finished |
| `achievements_total` | key | Achievements earned |
| `level_ups_total` | | Levels reached |
| `cosmetics_bought_total` | kind | Themes, tools, foils bought |
| `charms_bought_total` | kind, tier | Charms bought |
| `charm_uses_total` | kind | Charm charges spent |
| `charm_coins_total` | kind | Coins charms gave: taken off prices (discount), added to wins (payout) |
| `shouts_total`, `gifts_total` | | Shoutbox messages, tickets sent to others |
| `duels_total` | status (open, accepted, declined, cancelled, expired) | Duels |
| `pot_entries_total`, `pots_won_total` | | Daily pot entries, pots drawn with a winner |
| `guests_forgotten_total` | | Guest accounts deleted once no session could reach them |
| `live_sockets`, `online_users` | | Open live sockets, players with one (gauges) |
| `players` | kind (member, guest) | Accounts (gauge) |
| `db_pool_connections`, `db_pool_available`, `db_pool_waiting` | | Database pool (gauges) |
