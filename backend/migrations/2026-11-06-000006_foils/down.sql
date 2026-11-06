DELETE FROM cosmetics WHERE kind = 'foil';
ALTER TABLE cosmetics DROP CONSTRAINT cosmetics_kind_check;
ALTER TABLE cosmetics ADD CONSTRAINT cosmetics_kind_check CHECK (kind IN ('theme', 'tool'));
ALTER TABLE users DROP COLUMN foil;
