-- What a player has collected: series scratched, bundles bought, seals won under
CREATE TABLE collected (
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    -- series, bundle, seal
    kind TEXT NOT NULL,
    key TEXT NOT NULL,
    PRIMARY KEY (user_id, kind, key)
);

-- Runs of wins and losses, and hours with every quest round done
ALTER TABLE stats ADD COLUMN win_run INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats ADD COLUMN lose_run INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats ADD COLUMN best_win_run INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats ADD COLUMN best_lose_run INTEGER NOT NULL DEFAULT 0;
ALTER TABLE stats ADD COLUMN full_hours INTEGER NOT NULL DEFAULT 0;
