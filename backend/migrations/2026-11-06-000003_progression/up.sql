-- Experience: levels are worked out from it (progress/levels.rs)
ALTER TABLE users ADD COLUMN xp BIGINT NOT NULL DEFAULT 0;

-- Lifetime counters that achievements read
CREATE TABLE stats (
    user_id BIGINT PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    scratched INTEGER NOT NULL DEFAULT 0,
    wins INTEGER NOT NULL DEFAULT 0,
    best_win BIGINT NOT NULL DEFAULT 0,
    quests_done INTEGER NOT NULL DEFAULT 0
);

-- The day's quests, on the player's calendar day
CREATE TABLE quests (
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    day DATE NOT NULL,
    key TEXT NOT NULL,
    progress INTEGER NOT NULL DEFAULT 0,
    done_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, day, key)
);

-- Per player and day: the one reroll, and the all-done bonus
CREATE TABLE quest_days (
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    day DATE NOT NULL,
    rerolled BOOLEAN NOT NULL DEFAULT false,
    bonus_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, day)
);

CREATE TABLE achievements (
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    id TEXT NOT NULL,
    earned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, id)
);
