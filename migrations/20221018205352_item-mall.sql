CREATE TABLE user_item_mall (
    id serial constraint user_item_mall_pk primary key,
    user_id int constraint user_item_mall_user_fk references users,
    character_id int constraint user_item_mall_char_fk references characters,
    server_id int constraint user_item_mall_server_fk references servers,
    key varchar(255) not null,
    expiry timestamptz not null
);