-- Gift tickets: who sent it (the name kept for display)
ALTER TABLE tickets ADD COLUMN from_user_id BIGINT REFERENCES users (id) ON DELETE SET NULL;
ALTER TABLE tickets ADD COLUMN from_name TEXT;

-- The shared daily pot (UTC days) and its entries
CREATE TABLE pots (
    day DATE PRIMARY KEY,
    amount BIGINT NOT NULL DEFAULT 0,
    winner_id BIGINT REFERENCES users (id) ON DELETE SET NULL,
    drawn_at TIMESTAMPTZ
);
CREATE TABLE pot_entries (
    id BIGSERIAL PRIMARY KEY,
    day DATE NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX pot_entries_day_idx ON pot_entries (day);

-- Duels between two players: both stake, the higher roll takes the pot
CREATE TABLE duels (
    id BIGSERIAL PRIMARY KEY,
    challenger_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    opponent_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    stake BIGINT NOT NULL,
    -- open, accepted, declined, cancelled, expired
    status TEXT NOT NULL DEFAULT 'open',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    decided_at TIMESTAMPTZ,
    winner_id BIGINT REFERENCES users (id) ON DELETE SET NULL,
    CHECK (challenger_id <> opponent_id)
);
CREATE INDEX duels_open_idx ON duels (status, created_at);
