-- The leaderboard ranks members by coins and by experience
CREATE INDEX users_coins_rank_idx ON users (coins DESC, id) WHERE NOT is_guest;
CREATE INDEX users_xp_rank_idx ON users (xp DESC, id) WHERE NOT is_guest;
