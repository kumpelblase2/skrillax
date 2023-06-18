# Silkroad Protocol

This crate contains the actual definitions for the operations used by the silkroad client & server. These are based 
on a recent iSro (~v1.584 at the time of writing) of the game. These use the custom [serialization/deserialization 
crate](../silkroad-serde/README.md) to go to and from the binary representation used in the packets. The protocol 
definition tries to map the operation structures as good as possible to idiomatic Rust structures.