use silkroad_protocol::general::*;
use silkroad_protocol::login::*;
use skrillax_protocol::{define_inbound_protocol, define_outbound_protocol};

define_inbound_protocol! { GatewayClientProtocol =>
    KeepAlive,
    PatchRequest,
    IdentityInformation,
    GatewayNoticeRequest,
    LoginRequest,
    SecurityCodeInput,
    ShardListRequest,
    PingServerRequest
}

define_outbound_protocol! { GatewayServerProtocol =>
    PatchResponse,
    IdentityInformation,
    GatewayNoticeResponse,
    PasscodeRequiredResponse,
    PasscodeResponse,
    LoginResponse,
    SecurityCodeResponse,
    ShardListResponse,
    PingServerResponse
}
