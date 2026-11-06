# Scratch-me

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

## What is in so far (M1)

- Accounts (sign up, log in, log out), with the player's time zone
- Lucky Three: a 3×3 grid where three of a kind wins coins, another ticket or a pocket refill
- Pocket: one free ticket every 2 hours, holds 3
- Daily envelope: one per calendar day, with a 7-day streak cycle
- Scratch canvas: finger-speed brush, falling foil flakes, auto-reveal at 62 %
