use silkroad_gateway_protocol::*;
use skrillax_stream::registry::PacketRegistryBuilder;

pub(crate) trait GatewayPacketRegistryExt {
    fn register_gateway_packets(self) -> Self;
}

impl GatewayPacketRegistryExt for PacketRegistryBuilder {
    fn register_gateway_packets(self) -> Self {
        self.register_outgoing::<PatchResponse>()
            .register_outgoing::<GatewayNoticeResponse>()
            .register_outgoing::<PasscodeRequiredResponse>()
            .register_outgoing::<LoginResponse>()
            .register_outgoing::<SecurityCodeResponse>()
            .register_outgoing::<ShardListResponse>()
            .register_outgoing::<PingServerResponse>()
            .register_incoming::<PatchRequest>()
            .register_incoming::<GatewayNoticeRequest>()
            .register_incoming::<LoginRequest>()
            .register_incoming::<SecurityCodeInput>()
            .register_incoming::<ShardListRequest>()
            .register_incoming::<PingServerRequest>()
    }
}
