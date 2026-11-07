-- A ticket from a bundle carries its seal (tickets/seals.rs): its look and
-- what it does to the prize
ALTER TABLE tickets ADD COLUMN seal TEXT;
