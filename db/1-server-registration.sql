CREATE TABLE servers (
    id serial primary key,
    name varchar(255) not null constraint servers_uname unique,
    region varchar(3) not null default 'EU',
    address varchar(255) not null,
    port smallint not null default 22230,
    rpc_port smallint not null default 1337,
    token varchar(255) not null
);