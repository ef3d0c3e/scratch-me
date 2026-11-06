-- Themes and scratch tools owned. The defaults (midnight, coin) are owned by
-- everyone and never stored.
CREATE TABLE cosmetics (
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('theme', 'tool')),
    item TEXT NOT NULL,
    obtained_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, kind, item)
);

-- What the player has on
ALTER TABLE users ADD COLUMN theme TEXT NOT NULL DEFAULT 'midnight';
ALTER TABLE users ADD COLUMN tool TEXT NOT NULL DEFAULT 'coin';
