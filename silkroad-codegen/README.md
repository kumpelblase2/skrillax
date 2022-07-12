# Specification Codegen

This takes the specification from `/packet-spec` and generates rust structures, serialization, and deserialization code.
I chose this instead of just using rust structures directly as this could potentially allow reuse, even in other
languages. Even if I used `Derive` in rust I'd still have to write some (de-)serialization by hand because serde and the
like don't (and possibly can't)
support what I need.

There are currently 3 types of items it can handle:

- Enums
- Structs
- Packets

Packets are in themselves also just enums/structs, but they have an opcode and a direction assigned and will be added to
the general packet handler.

This project is mostly finished from my perspective, but there might be some things that are still necessary. For
example, the conditional logic is
very much just bolted on and may be rust specific. It's not used as often to warrant a proper design and implementation,
but still makes the goal of
reuse hard to achieve. It might also be nice to re-use enums/structs from other crates, like a common crate, so things
don't get duplicated. Or at
least providing from/to mappings in those cases.
