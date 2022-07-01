//! silkroad-security provides an implementation for the handshake and encryption used in the protocol of [Silkroad Online](http://www.silkroadonline.net/)
//! from the server side.
//! There is a module to help with decrypting passcode inputs from the client using the [passcode::PassCodeDecoder] struct, but
//! the main focus of this crate is the [security::SilkroadSecurity] struct. This is based in large parts upon
//! [pushedx's work](https://github.com/DummkopfOfHachtenduden/SilkroadDoc/wiki/Silkroad-Security). However, this crate
//! does not concern itself with how data is actually being sent and only operates on the bytes that have already been
//! taken from a packet stream or other source.
pub mod passcode;
pub mod security;
