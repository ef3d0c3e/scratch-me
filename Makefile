# Development commands. Copy .env.example to .env first.
.PHONY: dev db-up db-down api web test test-e2e check fmt prod-up prod-down prod-logs

dev: db-up
	@echo "API on :8081, web on http://localhost:5174"
	@trap 'kill 0' INT TERM; cargo run -p scratchme-api & (cd frontend && npm run dev) & wait

db-up:
	docker compose up -d --wait db

db-down:
	docker compose down

api: db-up
	cargo run -p scratchme-api

web:
	cd frontend && npm run dev

# Backend unit + integration tests (integration needs TEST_DATABASE_URL and the db)
test: db-up
	docker compose exec -T db psql -U scratchme -tc "SELECT 1 FROM pg_database WHERE datname='scratchme_test'" | grep -q 1 \
		|| docker compose exec -T db psql -U scratchme -c "CREATE DATABASE scratchme_test"
	cargo test
	cd frontend && npm run test:unit

# Browser tests (Playwright) on phone and desktop sizes, against the dev stack;
# starts the API and web server if needed. Uses the system Chromium when there is one.
test-e2e: db-up
	cd frontend && CHROMIUM_PATH="$${CHROMIUM_PATH:-$$(command -v chromium || true)}" npm run test:e2e

check:
	cargo clippy --all-targets -- -D warnings
	cargo fmt --check
	cd frontend && npm run check && npx prettier --check src e2e

fmt:
	cargo fmt
	cd frontend && npx prettier --write src e2e

# Production (compose.prod.yaml, behind the host's nginx); needs .env.prod, see docs/deploy.md
PROD = docker compose -f compose.prod.yaml --env-file .env.prod

prod-up:
	$(PROD) up -d --build

prod-down:
	$(PROD) down

prod-logs:
	$(PROD) logs -f --tail=100
