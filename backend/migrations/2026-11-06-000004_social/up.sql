-- The achievement a player shows next to their name (one they earned)
ALTER TABLE users ADD COLUMN title TEXT;
-- Admins post announcements; set with `scratchme-api make-admin <username>`
ALTER TABLE users ADD COLUMN is_admin BOOLEAN NOT NULL DEFAULT false;

CREATE TABLE shouts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX shouts_created_idx ON shouts (created_at DESC);
CREATE INDEX shouts_user_idx ON shouts (user_id, created_at DESC);

-- A player's shouts hidden from someone who muted them
CREATE TABLE mutes (
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    muted_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, muted_id),
    CHECK (user_id <> muted_id)
);

CREATE TABLE announcements (
    id BIGSERIAL PRIMARY KEY,
    author_id BIGINT REFERENCES users (id) ON DELETE SET NULL,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    removed_at TIMESTAMPTZ
);
