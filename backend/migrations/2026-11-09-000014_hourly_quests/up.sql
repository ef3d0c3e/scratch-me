-- Quests come by the hour now, three rounds each: a slot is the hour's start
-- (UTC). The daily quests are dropped (no progress worth keeping).
DROP TABLE quests;
DROP TABLE quest_days;

CREATE TABLE quests (
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    slot TIMESTAMPTZ NOT NULL,
    round INTEGER NOT NULL DEFAULT 1,
    key TEXT NOT NULL,
    progress INTEGER NOT NULL DEFAULT 0,
    done_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, slot, round, key)
);

-- Per player and hour: the one swap, and the rounds' bonuses paid
CREATE TABLE quest_slots (
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    slot TIMESTAMPTZ NOT NULL,
    rerolled BOOLEAN NOT NULL DEFAULT false,
    bonus_at TIMESTAMPTZ,
    bonuses INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, slot)
);
