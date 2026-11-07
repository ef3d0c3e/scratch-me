DROP TABLE quests;
DROP TABLE quest_slots;
CREATE TABLE quests (
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    day DATE NOT NULL,
    key TEXT NOT NULL,
    progress INTEGER NOT NULL DEFAULT 0,
    done_at TIMESTAMPTZ,
    round INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (user_id, day, round, key)
);
CREATE TABLE quest_days (
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    day DATE NOT NULL,
    rerolled BOOLEAN NOT NULL DEFAULT false,
    bonus_at TIMESTAMPTZ,
    bonuses INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, day)
);
