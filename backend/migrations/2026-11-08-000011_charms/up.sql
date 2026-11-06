-- Charms worn: one per kind, replaced by the next one bought once spent or expired
CREATE TABLE charms (
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    -- discount, luck, payout
    kind TEXT NOT NULL,
    -- bronze, silver, gold
    tier TEXT NOT NULL,
    charges INT NOT NULL CHECK (charges >= 0),
    bought_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (user_id, kind)
);
