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