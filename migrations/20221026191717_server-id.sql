alter table servers
    add column identifier smallint not null default 1;
alter table servers
    drop constraint servers_uname;
alter table servers
    add constraint server_uidentifier unique (identifier);