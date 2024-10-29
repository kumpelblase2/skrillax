use silkroad_protocol::auth::AuthProtocol;
use silkroad_protocol::character::{CharselectClientProtocol, CharselectServerProtocol};
use silkroad_protocol::chat::{ChatClientProtocol, ChatServerProtocol};
use silkroad_protocol::combat::{CombatClientProtocol, CombatServerProtocol};
use silkroad_protocol::community::{FriendListClientProtocol, FriendListServerProtocol};
use silkroad_protocol::general::BaseProtocol;
use silkroad_protocol::gm::{GmClientProtocol, GmServerProtocol};
use silkroad_protocol::inventory::{InventoryClientProtocol, InventoryServerProtocol};
use silkroad_protocol::movement::{MovementClientProtocol, MovementServerProtocol};
use silkroad_protocol::skill::{SkillClientProtocol, SkillServerProtocol};
use silkroad_protocol::world::{StatClientProtocol, StatServerProtocol, WorldClientProtocol, WorldServerProtocol};
use skrillax_protocol::{define_inbound_protocol, define_outbound_protocol};

define_inbound_protocol! { AgentClientProtocol =>
    +
    BaseProtocol,
    AuthProtocol,
    MovementClientProtocol,
    SkillClientProtocol,
    ChatClientProtocol,
    FriendListClientProtocol,
    CharselectClientProtocol,
    StatClientProtocol,
    CombatClientProtocol,
    WorldClientProtocol,
    InventoryClientProtocol,
    GmClientProtocol
}

define_outbound_protocol! { AgentServerProtocol =>
    +
    BaseProtocol,
    AuthProtocol,
    ChatServerProtocol,
    MovementServerProtocol,
    FriendListServerProtocol,
    CharselectServerProtocol,
    SkillServerProtocol,
    StatServerProtocol,
    CombatServerProtocol,
    WorldServerProtocol,
    InventoryServerProtocol,
    GmServerProtocol
}
