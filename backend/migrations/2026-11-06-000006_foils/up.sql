-- Ticket foils: a third kind of cosmetic, worn like themes and tools
ALTER TABLE users ADD COLUMN foil TEXT NOT NULL DEFAULT 'silver';
ALTER TABLE cosmetics DROP CONSTRAINT cosmetics_kind_check;
ALTER TABLE cosmetics ADD CONSTRAINT cosmetics_kind_check CHECK (kind IN ('theme', 'tool', 'foil'));
