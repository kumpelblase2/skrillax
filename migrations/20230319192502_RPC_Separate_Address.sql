ALTER TABLE servers
    ADD COLUMN rpc_address varchar(255);
UPDATE servers
set rpc_address = address;
ALTER TABLE servers
    ALTER COLUMN rpc_address SET NOT NULL;