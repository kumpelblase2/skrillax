CREATE TYPE race as enum ('chinese', 'european');
ALTER TABLE characters ADD COLUMN race race DEFAULT 'chinese';

UPDATE characters set race = case character_type > 2000 when true then 'european'::race else 'chinese'::race end;

ALTER TABLE characters ALTER COLUMN race SET NOT NULL;