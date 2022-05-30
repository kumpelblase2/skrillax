# Specification Codegen

This takes the specification from `/packet-spec` and generates rust structures, serialization, and deserialization code.
I chose this instead of just using rust structures directly as this could potentially allow reuse, even in other 
languages, I'd still have to write (de-)serialization by hand because serde and the like don't (and possibly can't) 
support what I need, and it's a lot simpler to edit.

There are currently 3 types of items it can handle:
- Enums
- Structs
- Packets

Packets are in themselves also just enums/structs, but they have an opcode and a direction assigned and will be added to
the general packet handler.

This project is mostly done from my perspective, but there might be some things that are still necessary. One of these 
things is some conditional logic in the parsing which is provided in the spec. In a few cases, a packet contents may 
vary depending on certain conditions of the previous data. An example would be if an area is a dungeon, the position 
gets presented differently (u32 instead of u16). So far, this has mostly been encountered on the server side, which I 
can work around without conditional logic, but in the future there may be client packets that require such a thing. It 
might also be nice to re-use enums/structs from an existing crate so things don't get duplicated.