CREATE TABLE hotbar_entries
(
    character_id INTEGER  NOT NULL REFERENCES characters (id),
    slot         SMALLINT NOT NULL,
    kind         SMALLINT NOT NULL,
    data         INTEGER  NOT NULL,
    CONSTRAINT PK_CHARACTER_SLOT PRIMARY KEY (character_id, slot)
);