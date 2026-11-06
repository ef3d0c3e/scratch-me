-- Treasure Dig: the spots dug so far, in order (the ticket stays unscratched
-- until the last dig)
ALTER TABLE tickets ADD COLUMN picks JSONB;
