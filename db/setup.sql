create type "FriendStatus" as enum ('REQUESTED', 'ACCEPTED', 'DECLINED');

create table users (
    id serial primary key,
    username varchar,
    password varchar,
    passcode varchar,
    invalid_passcode_count integer default 0
);

create table characters (
    id serial constraint characters_pk primary key,
    user_id integer not null constraint characters_users_id_fk references users on delete cascade,
    server_id integer,
    charname varchar not null,
    character_type integer not null,
    scale smallint not null,
    level smallint default 1 not null,
    max_level smallint default 1,
    exp bigint default 0 not null,
    strength smallint default 20 not null,
    intelligence smallint default 20 not null,
    stat_points smallint default 0 not null,
    current_hp integer default 200 not null,
    current_mp integer default 200 not null,
    deletion_end timestamp with time zone,
    sp integer default 0,
    x real default 0,
    y real default 0,
    z real default 0,
    region smallint,
    berserk_points smallint default 0,
    gold bigint default 0 not null,
    sp_exp integer default 0,
    beginner_mark boolean default true not null,
    gm boolean default false not null,
    last_logout timestamp with time zone,
    rotation smallint default 0
);

create index characters_user_id_server_id_charname_index on characters (user_id, server_id, charname);

create table character_items (
    id serial constraint character_items_pk primary key,
    character_id integer not null constraint character_items_characters_id_fk references characters on delete cascade,
    item_obj_id integer not null,
    upgrade_level smallint,
    slot smallint,
    variance bigint default 0
);

create index character_items_character_id_index on character_items (character_id);

create table character_masteries (
    character_id integer not null constraint character_masteries_characters_id_fk references characters on delete cascade,
    mastery_id integer not null,
    level smallint default 0 not null,
    constraint character_masteries_pk primary key (character_id, mastery_id)
);

create table friends_groups (
    id serial not null constraint friends_groups_pk primary key,
    character_id integer not null constraint friends_groups_characters_id_fk references characters,
    name varchar not null
);

create unique index friends_groups_character_id_group_name_uindex on friends_groups (character_id, name);

create table friends (
    id serial constraint friends_pk primary key,
    character_id integer not null constraint friends_characters_id_fk references characters,
    group_id integer,
    friend_id integer not null constraint friends_characters_id_fk_2 references characters,
    status "FriendStatus" default 'REQUESTED'::"FriendStatus"
);

create unique index friends_character_id_friend_id_uindex on friends (character_id, friend_id);

create table news (
    id serial constraint news_pk primary key,
    title varchar not null,
    body text not null,
    date timestamp with time zone default now() not null,
    visible boolean default false not null
);

create table user_servers (
    user_id integer not null constraint user_servers_users_id_fk references users on delete cascade,
    server_id integer not null,
    job smallint default 0 not null,
    premium_type smallint default 0 not null,
    premium_end timestamp with time zone
);

create unique index user_servers_user_id_server_id_uindex on user_servers (user_id, server_id);