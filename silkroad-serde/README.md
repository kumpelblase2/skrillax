# Silkroad Serde

[serde](https://serde.rs/) is a well known and respected serialization and deserialization framework for Rust. And 
while we _could_ make it work for our context, Silkroad is unfortunately a little annoying in a few respects, which 
would make working with `serde` a little cumbersome. Thus, this provides a more focused approach to 
serialization/deserialization in the Silkroad context.

Silkroad does a few things that make it hard to generally serialize the packets:
- They're not self describing - which is, however, expected
- Lists can be represented in different ways, depending on the specific operation
- Attributes may appear or be hidden depending on previous values
- Attributes may appear or be hidden depending on external information

For example, there are three types if list representations in Silkroad: Length based, Break based, or Continue based.
The first is quite obvious; a length is prepended and then all elements follow without any separator. "Break based" 
and "Continue based" are similar in that there's a separator in between each element which differs on the last 
element. Both use `1` as the separator, but will use `2` or `0` as the end element, respectively. Why do you need 
multiple ways to encode a list of values? I don't know. Other weirdness includes some strings using wide character 
(2 byte) encoding or the content completely changing depending on what 
type of item it belongs to.

This is not impossible to implement with serde, in fact, the implementation would be easy, but would be more 
exhausting to use given you'd have to constantly reach for `deserialize_with` and remembering the exact function that 
does what you want. Especially once you have to serialize/deserialize a field that depends on a previous field, 
you're going to have a hard time (Example: a `ChatMessage` packet has an optional field that depends on the type of 
message which has been encoded in a previous field). To make it easier to work with and make this quirks 
"implementable", this custom 'serde' exists.

## Silkroad Operation Serialization/Deserialization

Most of the serialization/deserialization logic is implemented in the [`-derive`](../silkroad-serde-derive/README.md)
crate. This crate only includes ser/de definitions for primitive values, such as time, `u8` and the like. However, 
I'd like to explain the basics of how data is serialized - in general - in the Silkroad world.

Let's begin with the basics: Silkroad encodes data (mostly) in little endian byte order, so a packet length of `256`,
which is a `u16`, will show up as `0x00 0x01` instead of `0x01 0x00`. There are no separators between data fields, 
so values are present directly adjacent to each other. If you have three fields of varying size, the serialized size 
will always match the total space the fields would use up in memory. Arrays of constant length also encode only the 
data it contains (the number of elements is know and so is the size of each element). Varying length lists (or in 
Rust terms: `Vec`s) have, as previously mentioned, three different ways of being encoded. Either the length is 
prepended, which is usually of size `u8`, but may also be `u16`, and then all elements will follow without any 
further separators. Alternatively, a `u8` is used to show if there's more (with a value of `1`) followed by the next 
element or if it's the end with either `0` or a `2`, depending on where it's used. Enum-like elements (for example, 
error codes or, generally, different variants of something) often contain a "variant id" denoted as a `u8`, followed 
by the specific fields for that variant. As such, variants that don't contain any data would only have the variant 
id and nothing else. Lastly, there are optional fields, which contain a `u8` specifying if data is present (value of 
`1`) or no data (value of `0`). Everything else uses any combination of the previously defined atoms.