-- Lifetime counts the later achievements read: coins won, bundles bought
ALTER TABLE stats ADD COLUMN coins_won BIGINT NOT NULL DEFAULT 0;
ALTER TABLE stats ADD COLUMN bundles INTEGER NOT NULL DEFAULT 0;
