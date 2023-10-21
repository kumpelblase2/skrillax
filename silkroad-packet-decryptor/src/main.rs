use byteorder::ByteOrder;
use clap::{arg, ArgAction};
use log::{debug, error, LevelFilter};
use pcap_file::pcap::{PcapPacket, PcapReader, PcapWriter};
use pcap_file::PcapError;
use pktparse::ethernet::EtherType;
use pktparse::ip::IPProtocol;
use pktparse::ipv4::IPv4Header;
use pktparse::ipv6::IPv6Header;
use pktparse::tcp::TcpHeader;
use pktparse::{ethernet, ipv4, ipv6, tcp};
use silkroad_security::security::SilkroadSecurity;
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::path::Path;

fn g_pow_x_mod_p(p: i64, mut x: u32, g: u32) -> u32 {
    let mut current: i64 = 1;
    let mut mult: i64 = g as i64;

    while x != 0 {
        if (x & 1) > 0 {
            current = (mult * current) % p;
        }
        x >>= 1;
        mult = (mult * mult) % p;
    }
    current as u32
}

struct DecryptionOrchestrator(u8);

impl DecryptionOrchestrator {
    pub fn new(threads: u8) -> Self {
        Self(threads)
    }

    pub fn break_security(&self, input: &SecurityData, client_b: u32, client_key: u64) -> Option<SilkroadSecurity> {
        debug!("Trying to crack key exchange with {} threads...", self.0);
        let options = self.find_x(input.p, input.g, input.a);
        for option in &options {
            let mut security = SilkroadSecurity::default();
            security.initialize_with(0, 0, input.handshake_bytes, *option, input.p, input.a);
            match security.start_challenge(client_b, client_key) {
                Ok(_) => {
                    security.accept_challenge().unwrap();
                    debug!("Checking candidate {}... Success!", option);
                    return Some(security);
                },
                _ => {
                    debug!("Checking candidate {}... Fail", option);
                },
            }
        }

        error!(
            "Could not break security. None of the {} options worked.",
            options.len()
        );

        None
    }

    fn find_x(&self, value_p: u32, value_g: u32, value_a: u32) -> Vec<u32> {
        let thread_count = self.0 as u32;
        let steps = (u32::MAX / 2) / thread_count;
        let mut results = Vec::new();

        let mut threads = Vec::new();

        for thread in 0..thread_count {
            threads.push(std::thread::spawn(move || {
                let start = thread * steps;
                let end = (thread + 1) * steps;
                (start..end)
                    .rev()
                    .find(|&i| g_pow_x_mod_p(value_p as i64, i, value_g) == value_a)
            }));
        }

        for thread in threads {
            if let Some(number) = thread.join().unwrap() {
                results.push(number);
            }
        }

        results
    }
}

enum IpHeader {
    IPv4(IPv4Header),
    IPv6(IPv6Header),
}

impl IpHeader {
    fn protocol(&self) -> IPProtocol {
        match &self {
            IpHeader::IPv4(header) => header.protocol,
            IpHeader::IPv6(header) => header.next_header,
        }
    }
}

struct SecurityData {
    handshake_bytes: u64,
    g: u32,
    p: u32,
    a: u32,
}

struct Rewriter {
    read: RefCell<PcapReader<File>>,
    write: RefCell<PcapWriter<File>>,
    server_ports: Vec<u16>,
    filter_other: bool,
    decryption: DecryptionOrchestrator,
    current_security: RefCell<Option<SilkroadSecurity>>,
    security_initialization: RefCell<Option<SecurityData>>,
}

impl Rewriter {
    pub fn new(
        read: PcapReader<File>,
        write: PcapWriter<File>,
        server_ports: Vec<u16>,
        filter_other: bool,
        decryption: DecryptionOrchestrator,
    ) -> Self {
        Self {
            read: RefCell::new(read),
            write: RefCell::new(write),
            server_ports,
            decryption,
            filter_other,
            current_security: RefCell::new(None),
            security_initialization: RefCell::new(None),
        }
    }

    fn get_tcp_data(data: &[u8]) -> Option<(TcpHeader, &[u8])> {
        if let Ok((remaining, ethernet_frame)) = ethernet::parse_ethernet_frame(data) {
            if let Ok((remaining, ip_header)) = match ethernet_frame.ethertype {
                EtherType::IPv4 => ipv4::parse_ipv4_header(remaining).map(|(rem, ip)| (rem, IpHeader::IPv4(ip))),
                EtherType::IPv6 => ipv6::parse_ipv6_header(remaining).map(|(rem, ip)| (rem, IpHeader::IPv6(ip))),
                _ => return None,
            } {
                if matches!(ip_header.protocol(), IPProtocol::TCP) {
                    if let Ok((remaining, tcp_header)) = tcp::parse_tcp_header(remaining) {
                        return Some((tcp_header, remaining));
                    }
                }
            }
        }
        None
    }

    fn is_packet_encrypted(data: &[u8]) -> bool {
        (data[1] & 0x80) != 0
    }

    fn handle_unencrypted(&self, tcp: &TcpHeader, data: &[u8]) {
        let opcode = byteorder::LittleEndian::read_u16(&data[2..4]);
        if self.server_ports.contains(&tcp.source_port) {
            // S -> C
            if opcode == 0x5000 {
                // Handshake Packet
                if data[0] == 0x25 {
                    // if size = 0x25
                    let handshake_bytes = byteorder::LittleEndian::read_u64(&data[23..=30]);
                    let g = byteorder::LittleEndian::read_u32(&data[31..=34]);
                    let p = byteorder::LittleEndian::read_u32(&data[35..=38]);
                    let a = byteorder::LittleEndian::read_u32(&data[39..=42]);
                    let init = SecurityData {
                        handshake_bytes,
                        g,
                        p,
                        a,
                    };
                    debug!("Server handshake start encountered.");
                    *self.security_initialization.borrow_mut() = Some(init);
                }
            }
        } else {
            // C -> S
            if opcode == 0x5000 {
                debug!("Client handshake challenge encountered.");
                let b = byteorder::LittleEndian::read_u32(&data[6..10]);
                let key = byteorder::LittleEndian::read_u64(&data[10..18]);
                let init = self
                    .security_initialization
                    .borrow_mut()
                    .take()
                    .expect("No handshake received from server yet.");

                let result = self.decryption.break_security(&init, b, key);
                *self.current_security.borrow_mut() = result;
            }
        }
    }

    fn handle_packet(&self, packet: PcapPacket, tcp: &TcpHeader, mut data: Vec<u8>) -> PcapPacket {
        let header_len = packet.data.len() - data.len();
        let mut header_data = packet.data[0..header_len].to_vec();
        if Self::is_packet_encrypted(&data) {
            if let Some(ref security) = *self.current_security.borrow() {
                security
                    .decrypt_mut(&mut data[2..])
                    .expect("Security should have been initialized.");
                data[1] &= !0x80
            }
        } else {
            self.handle_unencrypted(tcp, data.as_slice());
        }
        header_data.append(&mut data);
        PcapPacket::new_owned(packet.timestamp, packet.orig_len, header_data)
    }

    fn should_handle_packet(&self, tcp: &TcpHeader) -> bool {
        self.server_ports.contains(&tcp.source_port) || self.server_ports.contains(&tcp.dest_port)
    }

    pub fn run(&self) -> Result<(), PcapError> {
        while let Some(packet) = self.read.borrow_mut().next_packet() {
            let packet = match packet {
                Ok(p) => p,
                Err(e) => {
                    error!("Skipping malformed packet: {:?}", e);
                    continue;
                },
            };
            if let Some((tcp, data)) = Self::get_tcp_data(&packet.data) {
                if self.should_handle_packet(&tcp) {
                    if tcp.flag_psh && !data.is_empty() {
                        // How to deal with packets that are split?
                        // We currently can't turn headers back into their bytes
                        // so combining two packets is not feasible.
                        let mut data_copy = Vec::with_capacity(data.len());
                        for by in data {
                            data_copy.push(*by);
                        }
                        let result = self.handle_packet(packet.clone(), &tcp, data_copy);
                        self.write.borrow_mut().write_packet(&result)?;
                    } else {
                        self.write.borrow_mut().write_packet(&packet)?;
                    }
                    continue;
                }
            }

            if !self.filter_other {
                self.write.borrow_mut().write_packet(&packet)?;
            }
        }

        Ok(())
    }
}

fn main() -> Result<(), io::Error> {
    let cmd = clap::Command::new("silkroad-packet-decryptor")
        .bin_name("silkroad-packet-decryptor")
        .arg(arg!([pcap] "PCAP-file to decrypt.").required(true))
        .arg(arg!([port] "Game server port.").value_parser(clap::value_parser!(u16).range(1..)))
        .arg(
            arg!(-t --threads <COUNT> "Sets the threads to use. Defaults to half the threads available ot the system.")
                .value_parser(clap::value_parser!(u8).range(1..)),
        )
        .arg(arg!(-f --filter "Filters out unrelated packets").action(ArgAction::SetTrue))
        .arg(arg!(-v --verbose "Enables verbose output").action(ArgAction::SetTrue));

    let matches = cmd.get_matches();

    let file = matches.get_one::<String>("pcap").expect("A PCAP file should be given.");
    let threads = matches
        .get_one::<u8>("threads")
        .copied()
        .unwrap_or(num_cpus::get_physical() as u8);
    let decryption_orchestrator = DecryptionOrchestrator::new(threads);
    let port = matches.get_one::<u16>("port").copied().unwrap_or(15779);
    let ports = vec![15779, port];
    let verbose = matches.get_one::<bool>("verbose").copied().unwrap_or(false);
    let filter_level = if verbose { LevelFilter::Debug } else { LevelFilter::Info };
    env_logger::builder().filter_level(filter_level).init();
    let filter_other = matches.get_one::<bool>("filter").copied().unwrap_or(false);

    let file_in_path = Path::new(file.as_str());
    let file_in_dir = file_in_path
        .parent()
        .ok_or(io::Error::new(ErrorKind::NotFound, "Parent folder not found."))?;
    let file_in_name = file_in_path
        .file_stem()
        .ok_or(io::Error::new(ErrorKind::InvalidInput, "Filename is invalid."))?;
    let output_file = file_in_dir.join(format!("{}-decrypted.pcap", file_in_name.to_str().unwrap()));

    let file_in = File::open(file)?;
    let file_out = File::create(output_file)?;
    let pcap_reader = PcapReader::new(file_in).unwrap();
    let pcap_writer = PcapWriter::new(file_out).unwrap();

    let rewriter = Rewriter::new(pcap_reader, pcap_writer, ports, filter_other, decryption_orchestrator);
    match rewriter.run() {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Encountered an error: {:?}", e)
        },
    }

    Ok(())
}
