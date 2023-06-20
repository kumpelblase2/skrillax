# Skrillax

Learning Rust and ECS by implementing an emulator for an MMORPG.

Skrillax is my learning project for playing around with Rust, learning about lifetimes, shared state, async, and 
whatever else I encounter on the way. My goal isn't to have a (fully) working server emulator. However, having a 
somewhat working program at the end of the day would help with motivation.

This project is organized in many subprojects, each having their own individual goal:

- [silkroad-protocol](silkroad-protocol/README.md): Packet specification that is used for communicating with the client
- silkroad-rpc: Shared types for RPC between servers
- [silkroad-security](silkroad-security/README.md): Implementation of security primitives used in Silkroad
- silkroad-network: Abstraction to handle connections to Silkroad clients
- silkroad-navmesh: Navigation Mesh implementation, loading from official data files
- [silkroad-gateway](silkroad-gateway/README.md): Loginserver implementation
- [silkroad-agent](silkroad-agent/README.md): Gameserver implementation
- [silkroad-packet-decryptor](silkroad-packet-decryptor/README.md): Tool to decrypt encrypted packet stream from
  silkroad.
- [silkroad-serde](silkroad-serde/README.md): Serialization/Deserialization traits used for packets.
- [silkroad-serde-derive](silkroad-serde-derive/README.md): Derive macros to implement serialization/deserialization traits.

## Usage

While there's a lot (probably 97%) still missing, it's possible to run the server(s) and connect to it. There are 
two ways to go from here: Either you run the servers in a container or you run them directly. Please follow the 
steps for the method you chose below.

In both cases you also need a working Silkroad Online installation. This needs to also be accessible by the server, 
as this provides most of the data (skills, characters, items, world mesh).

### via Docker

If you want to run the servers in a docker container, ensure docker is already installed as well as docker-compose. 
Next, copy the `docker-compose.yml.example` to `docker-compose.yml`. Now adjust the `<silkroad game location>` to 
match the path of your silkroad directory (e.g. `/home/me/games/silkroad`) and `<configuration location>` to be the 
path to the `configs` dir in this repository (e.g. `/home/me/projects/skrillax/configs`). Notice there are two 
instances of the configuration location you have to fill in.

Before we can start the compose file, please create the docker image first, as it's used by both game and agent 
server. To do this, run the following command:
```shell
$ docker build -t kumpelblase2/skrillax .
```
This might take a while, depending on your system and internet connection, as this downloads all necessary 
dependencies as well as build the binary.

After this has completed, you can run:
```shell
$ docker-compose up
```
to start all servers; the database, the gateway server, and the agent server. You're almost ready, please continue 
with the [after install](#after-setup) step.

### Local Build

You can also choose to build and run the servers directly. For this you need to first install 
[Rust](https://www.rust-lang.org/tools/install). You can verify the installation by running `rustc --version` and 
checking that you get a version back. Additionally, make sure you have either installed or otherwise access to a 
PostgreSQL instance.

First, lets set up the database. For this, create a new database in postgres that we can use. Then migrate the 
database using the scripts provided in the `migrations` folder. You can either do this manually or, if you want to 
continuously develop, use `sqlx` to do it for you. You can install `sqlx` by running 
`cargo install sqlx-cli --features postgres`. Next, copy the `.env.example` to `.env` and adjust the connection 
string to your installation. Format is: `postgres://<user>:<password>@<host>/<db>`. Then, you can run `cargo sqlx 
migrate run`. After all this, your database should have some tables in it, albeit empty.

Now we need to adjust the configuration files. In `configs/agent_server.toml` and `configs/gateway_server.toml` 
configure the database connection similar to the following with the appropriate values for your environment:
```toml
[database]
host = "localhost"
user = "skrillax"
password = "skrillax"
database = "skrillax"
```
We also need to adjust the `game.data-location` path in the `agent_server.toml` to match that of your Silkroad 
installation directory as well as the `rpc-address` in the same config to be `localhost`. If you plan on running the 
agent and gateway server on different hosts, the address you put in here should be the host or ip the gateway can 
use to access the agent server.

With the configuration and database set up, we can now start the servers. It doesn't really matter in which order we 
do it - the gateway server will pick up the agent server by checking occasionally - but you can start the gateway 
server first:
```shell
$ cargo run --bin silkroad-gateway --release
```
After that, you can do the same with the agent server:
```shell
$ cargo run --bin silkroad-agent --release
```

Both servers should start up just fine and would now wait for connections.

### After Setup

The servers are now running, and you could connect to them, but there would be no user to use to log in. One can be 
created using the gateway:
```shell
$ cargo run --bin silkroad-gateway --release -- register <username> <password>
```
and fill in the parameters with the values you wish. With these, you will be able to log in.

As a last step, you need to configure Silkroad to use the local gateway server instead. You can do this either by 
using a so-called "Loader" to redirect the client to the gateway server on startup, or by editing the `Media.pk2` 
directly. 