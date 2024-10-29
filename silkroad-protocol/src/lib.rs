pub mod auth;
pub mod character;
pub mod chat;
pub mod combat;
pub mod community;
pub mod error;
pub mod general;
pub mod gm;
pub mod inventory;
pub mod login;
pub mod movement;
pub mod skill;
pub mod spawn;
pub mod world;

use crate::auth::AuthProtocol;
use crate::general::BaseProtocol;
use skrillax_protocol::define_protocol_enum;
pub use skrillax_serde::SilkroadTime;

define_protocol_enum! { SilkroadProtocol =>
    +
    BaseProtocol,
    AuthProtocol
}
