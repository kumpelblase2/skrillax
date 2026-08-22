use etherparse::{NetSlice, SlicedPacket, TransportSlice};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::net::{IpAddr, SocketAddr};

#[derive(Clone, Copy, Debug, Eq, PartialOrd, PartialEq)]
pub(crate) enum Direction {
    ServerToClient,
    ClientToServer,
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServerToClient => f.write_str("S->C"),
            Self::ClientToServer => f.write_str("C->S"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ConnectionKey {
    server: SocketAddr,
    client: SocketAddr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MatchedPacket {
    pub(crate) connection: ConnectionKey,
    pub(crate) direction: Direction,
}

/// Selects gateway and agent TCP traffic and normalizes its connection and direction.
pub(crate) struct CaptureFilter {
    gateway_port: u16,
    configured_agent_port: Option<u16>,
    learned_agent_endpoints: HashSet<SocketAddr>,
}

impl CaptureFilter {
    pub(crate) fn new(gateway_port: u16, configured_agent_port: Option<u16>) -> Self {
        Self {
            gateway_port,
            configured_agent_port,
            learned_agent_endpoints: HashSet::new(),
        }
    }

    /// Adds every distinct successful login target. Captures may contain
    /// repeated logins or logins for more than one agent server.
    pub(crate) fn add_agent_endpoint(&mut self, endpoint: SocketAddr) -> bool {
        self.learned_agent_endpoints.insert(endpoint)
    }

    pub(crate) fn match_packet(&self, packet: &SlicedPacket) -> Option<MatchedPacket> {
        let (source, destination) = socket_addresses(packet)?;

        let direction = if self.learned_agent_endpoints.contains(&source) {
            Direction::ServerToClient
        } else if self.learned_agent_endpoints.contains(&destination) {
            Direction::ClientToServer
        } else if source.port() == self.gateway_port || self.configured_agent_port == Some(source.port()) {
            Direction::ServerToClient
        } else if destination.port() == self.gateway_port || self.configured_agent_port == Some(destination.port()) {
            Direction::ClientToServer
        } else {
            return None;
        };

        let connection = match direction {
            Direction::ServerToClient => ConnectionKey {
                server: source,
                client: destination,
            },
            Direction::ClientToServer => ConnectionKey {
                server: destination,
                client: source,
            },
        };

        Some(MatchedPacket { connection, direction })
    }
}

fn socket_addresses(packet: &SlicedPacket) -> Option<(SocketAddr, SocketAddr)> {
    let (source_ip, destination_ip): (IpAddr, IpAddr) = match packet.net.as_ref()? {
        NetSlice::Ipv4(ipv4) => (
            ipv4.header().source_addr().into(),
            ipv4.header().destination_addr().into(),
        ),
        NetSlice::Ipv6(ipv6) => (
            ipv6.header().source_addr().into(),
            ipv6.header().destination_addr().into(),
        ),
        NetSlice::Arp(_) => return None,
    };
    let TransportSlice::Tcp(tcp) = packet.transport.as_ref()? else {
        return None;
    };

    Some((
        SocketAddr::new(source_ip, tcp.source_port()),
        SocketAddr::new(destination_ip, tcp.destination_port()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use etherparse::PacketBuilder;

    fn tcp_packet(source_ip: [u8; 4], source_port: u16, destination_ip: [u8; 4], destination_port: u16) -> Vec<u8> {
        let builder = PacketBuilder::ethernet2([1; 6], [2; 6])
            .ipv4(source_ip, destination_ip, 20)
            .tcp(source_port, destination_port, 1, 1024);
        let mut bytes = Vec::with_capacity(builder.size(0));
        builder.write(&mut bytes, &[]).unwrap();
        bytes
    }

    fn match_tcp_packet(filter: &CaptureFilter, bytes: &[u8]) -> Option<MatchedPacket> {
        let packet = SlicedPacket::from_ethernet(bytes).unwrap();
        filter.match_packet(&packet)
    }

    #[test]
    fn gateway_filter_matches_both_directions_by_port() {
        let filter = CaptureFilter::new(15779, None);
        let server_packet = tcp_packet([10, 0, 0, 2], 15779, [10, 0, 0, 1], 40000);
        let client_packet = tcp_packet([10, 0, 0, 1], 40000, [10, 0, 0, 2], 15779);

        let server_match = match_tcp_packet(&filter, &server_packet).unwrap();
        let client_match = match_tcp_packet(&filter, &client_packet).unwrap();

        assert_eq!(server_match.direction, Direction::ServerToClient);
        assert_eq!(client_match.direction, Direction::ClientToServer);
        assert_eq!(server_match.connection, client_match.connection);
    }

    #[test]
    fn configured_agent_port_is_filtered_without_a_login_response() {
        let filter = CaptureFilter::new(15779, Some(15780));
        let packet = tcp_packet([10, 0, 0, 1], 40000, [10, 0, 0, 2], 15780);

        let matched = match_tcp_packet(&filter, &packet).unwrap();

        assert_eq!(matched.direction, Direction::ClientToServer);
        assert_eq!(matched.connection.server, "10.0.0.2:15780".parse().unwrap());
    }

    #[test]
    fn learned_agent_endpoints_are_exact_and_idempotent() {
        let mut filter = CaptureFilter::new(15779, None);
        let first_endpoint = "10.0.0.2:15780".parse().unwrap();
        let second_endpoint = "10.0.0.3:15781".parse().unwrap();

        assert!(filter.add_agent_endpoint(first_endpoint));
        assert!(!filter.add_agent_endpoint(first_endpoint));
        assert!(filter.add_agent_endpoint(second_endpoint));

        let first = tcp_packet([10, 0, 0, 2], 15780, [10, 0, 0, 1], 40000);
        let second = tcp_packet([10, 0, 0, 1], 40001, [10, 0, 0, 3], 15781);
        let same_port_different_ip = tcp_packet([10, 0, 0, 4], 15780, [10, 0, 0, 1], 40002);

        assert_eq!(
            match_tcp_packet(&filter, &first).unwrap().direction,
            Direction::ServerToClient
        );
        assert_eq!(
            match_tcp_packet(&filter, &second).unwrap().direction,
            Direction::ClientToServer
        );
        assert!(match_tcp_packet(&filter, &same_port_different_ip).is_none());
    }
}
