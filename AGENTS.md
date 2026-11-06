<!-- lific:begin -->
## Issue tracking: Lific

This project tracks its work in Lific, as project `SCRAT` (https://lific.pundalik.org/SCRAT/overview).

> The tracker database was lost and repopulated from git history on 2026-10-24, so issue numbers cited in older commit messages use the previous numbering; each recreated issue notes its original id in its description.

Use the Lific MCP tools for issues, pages, and plans when they are connected. Use the `lific` CLI for work that needs this machine. The MCP server can run on another host, so it cannot see your files, working directory, or git history. The CLI also has every data command, for when MCP is not connected. Add `--json` for machine-readable output:

```bash
lific issue list --project SCRAT --json      # browse issues
lific issue get SCRAT-1                       # one issue with relations
lific issue update SCRAT-1 --status done      # close an issue
lific search "auth flow" --project SCRAT      # full-text search
```

Rules:
- When you finish an issue, set its status to `done`.
- File each issue in the right module, and keep it to one change.
- Use a plan for work that spans sessions.
- An issue assigned to a person (`for:` in lists) is theirs. Take only unassigned issues.
<!-- lific:end -->

## Development

- Stack and many modules come from AnyCardz (`../anycardz`). Before writing shared infrastructure (shoutbox, live WebSocket, achievements, missions, push), port it from there.
- `make dev` runs Postgres (Docker, host port 5434), the API (:8081) and the SvelteKit dev server (:5174). Run `make test` and `make check` before finishing a change.
- The server holds every outcome. A ticket's layout and prize are drawn when it is issued (`backend/src/tickets/`) and only sent by `POST /tickets/{id}/scratch`, which credits the prize in the same transaction. The client never decides a prize.
- Game rules are pure modules with no I/O (`tickets/`, `pocket.rs`, `envelope.rs`). Routes do the database work. Each series has an expected-value test: bought tickets return 70–90 % of their price.
- SvelteKit 3: `$lib` is the `#lib` subpath import and needs file extensions (`#lib/api.ts`), and hook types come from `@sveltejs/kit/hooks`.
- diesel-async 0.9: `conn.transaction(async move |conn| { ... })`.
- Shared client state goes through Svelte context, never module-level `$state`.
- UI strings live in `frontend/src/lib/i18n/{en,fr}.ts`. Add every new key to both; a test checks this.

## Tests

- Backend rules: unit tests next to the code. Endpoints: `backend/tests/api.rs` (real Postgres; run races with `tokio::join!`).
- Frontend logic: keep it out of components, in plain modules (e.g. `src/lib/scratch/foil.ts`), with Vitest tests next to it.
- Playwright reuses an API and web server already listening on :8081/:5174, even one left over from an earlier run and built from older code. After backend changes, stop leftovers first (`ss -ltnp | grep -E ':8081|:5174'`): a stale API shows up as every new flow failing at once.
- Flows: Playwright specs in `frontend/e2e/`, run on a phone and a desktop project. Use `newPlayer()` and `visit()` from `e2e/helpers.ts`, and prefer roles and visible text in locators.
- CI (`.github/workflows/ci.yml`, GitHub Actions) runs `make check`'s checks, `cargo test` with the API tests, Vitest, and the flows against the production build. Reproduce its flows with `cd frontend && CI=1 E2E_BUILD=1 npx playwright test` (one retry, so a flaky flow shows as `flaky`: fix it, do not lean on the retry).
