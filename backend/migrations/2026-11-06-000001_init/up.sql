CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    username TEXT NOT NULL,
    email TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    coins BIGINT NOT NULL DEFAULT 0 CHECK (coins >= 0),
    -- IANA time zone: the daily envelope follows the player's calendar day
    tz TEXT NOT NULL DEFAULT 'UTC',
    -- Pocket: free tickets held as of pocket_at; one more every refill period
    -- after pocket_at, up to the pocket size (see pocket.rs)
    pocket_count INTEGER NOT NULL DEFAULT 0,
    pocket_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- Names and addresses are unique whatever their case
CREATE UNIQUE INDEX users_username_key ON users (lower(username));
CREATE UNIQUE INDEX users_email_key ON users (lower(email));

CREATE TABLE sessions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    -- SHA-256 of the cookie token, the token itself is never stored
    token_hash BYTEA NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    user_agent TEXT NOT NULL DEFAULT ''
);
CREATE INDEX sessions_user_id_idx ON sessions (user_id);

-- The outcome is drawn when a ticket is issued: layout and prize are stored
-- and only sent to the player when they scratch it.
CREATE TABLE tickets (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    series TEXT NOT NULL,
    layout JSONB NOT NULL,
    prize JSONB NOT NULL,
    -- Where it came from: pocket, envelope, prize, shop
    source TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    scratched_at TIMESTAMPTZ
);
CREATE INDEX tickets_unscratched_idx ON tickets (user_id, id) WHERE scratched_at IS NULL;

-- Daily envelope
CREATE TABLE streaks (
    user_id BIGINT PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    -- Days in a row, as of last_claim
    day INTEGER NOT NULL DEFAULT 0,
    best INTEGER NOT NULL DEFAULT 0,
    -- The player's local date of the last claim
    last_claim DATE
);
