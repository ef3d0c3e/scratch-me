ALTER TABLE quest_days DROP COLUMN bonuses;
DELETE FROM quests WHERE round > 1;
ALTER TABLE quests DROP CONSTRAINT quests_pkey;
ALTER TABLE quests DROP COLUMN round;
ALTER TABLE quests ADD PRIMARY KEY (user_id, day, key);
