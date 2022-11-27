ALTER TABLE user_servers
    ADD CONSTRAINT user_servers_servers_id_fk FOREIGN KEY (server_id) REFERENCES servers (id);
ALTER TABLE user_servers
    ADD PRIMARY KEY (user_id, server_id);