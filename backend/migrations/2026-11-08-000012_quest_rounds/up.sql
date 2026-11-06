-- Rounds of quests: finishing a day's three opens three harder ones, up to four a day
ALTER TABLE quests ADD COLUMN round INTEGER NOT NULL DEFAULT 1;
ALTER TABLE quests DROP CONSTRAINT quests_pkey;
ALTER TABLE quests ADD PRIMARY KEY (user_id, day, round, key);

-- How many rounds' bonuses are paid (bonus_at: the last one)
ALTER TABLE quest_days ADD COLUMN bonuses INTEGER NOT NULL DEFAULT 0;
UPDATE quest_days SET bonuses = 1 WHERE bonus_at IS NOT NULL;
