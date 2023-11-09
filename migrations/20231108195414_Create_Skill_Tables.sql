create table character_skills
(
    skill_group_id     integer not null,
    level smallint not null default 1,
    character_id integer not null
        constraint character_skills_characters_id_fk
            references characters ON DELETE CASCADE,
    constraint character_skills_pk
        unique (skill_group_id, character_id)
);