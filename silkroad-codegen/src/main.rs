use crate::code_container::CodeContainer;
use crate::context::Context;
use crate::definition::{ContentEntry, PacketCollection, PacketReference, PacketSource};
use quick_xml::Reader;
use std::env;
use std::fs;
use std::path::Path;

mod code_container;
mod context;
mod definition;
mod reader;
mod structure;
mod stuct_like;
mod writer;

static PACKET_FILE_HEADER: &str = include_str!("file_header.in");
static READER_VAR_NAME: &str = "data_reader";
static BUFFER_VAR_NAME: &str = "data_writer";

fn main() {
    let args: Vec<String> = env::args().collect();
    let spec = args.get(1).expect("No source spec folder given");
    let output = args.get(2).expect("No output folder given");

    println!("Source: {}", spec);
    println!("Output: {}", output);
    let output_path = Path::new(output);

    let source_path = Path::new(spec);
    let files = fs::read_dir(source_path).unwrap();
    let mut all_client_packets = Vec::new();
    let mut all_server_packets = Vec::new();
    let mut modules = Vec::new();
    for file in files {
        let file = file.unwrap();
        if file.file_type().unwrap().is_file() && file.file_name().to_str().unwrap().ends_with(".xml") {
            println!("Generating for file: {}", file.file_name().to_str().unwrap());
            let (collection, mut client_packets, mut server_packets) = generate_packets_from(file.path(), &output_path);
            all_client_packets.append(&mut client_packets);
            all_server_packets.append(&mut server_packets);
            modules.push(collection);
        }
    }

    generate_library_root(modules, all_client_packets, all_server_packets, output_path);
}

fn generate_library_root(
    modules: Vec<String>,
    client_packets: Vec<PacketReference>,
    server_packets: Vec<PacketReference>,
    output_path: &Path,
) {
    let output_file = output_path.join("lib.rs");
    let mut scope = codegen::Scope::new();

    scope.import("bytes", "Bytes");
    scope.import("crate::error", "ProtocolError");

    for module in modules.iter() {
        scope.import(&format!("crate::{}", module), "*");
    }

    scope.raw("pub mod error;");
    for module in modules.iter() {
        scope.raw(&format!("pub mod {};", &module));
    }

    let client_enum = scope.new_enum("ClientPacket").vis("pub");
    for packet in client_packets.iter() {
        client_enum.new_variant(&packet.name).tuple(&packet.name);
    }

    let client_enum_deserialize = scope
        .new_impl("ClientPacket")
        .new_fn("deserialize")
        .vis("pub")
        .arg("opcode", "u16")
        .arg("data", "Bytes")
        .ret("Result<ClientPacket, ProtocolError>");

    client_enum_deserialize.attach_block(|| {
        let mut block = codegen::Block::new("match opcode");

        for packet in client_packets.iter() {
            block.line(&format!(
                "{} => Ok(ClientPacket::{}(data.try_into()?)),",
                &packet.opcode, &packet.name
            ));
        }

        block.line("_ => Err(ProtocolError::UnknownOpcode(opcode))");

        block
    });

    let server_enum = scope.new_enum("ServerPacket").vis("pub");
    for packet in server_packets.iter() {
        server_enum.new_variant(&packet.name).tuple(&packet.name);
    }

    let server_enum_impl = scope.new_impl("ServerPacket");
    let server_enum_serialize = server_enum_impl
        .new_fn("into_serialize")
        .vis("pub")
        .arg_self()
        .ret("(u16, Bytes)");

    server_enum_serialize.attach_block(|| {
        let mut block = codegen::Block::new("match self");

        for packet in server_packets.iter() {
            block.line(&format!(
                "ServerPacket::{}(data) => ({}, data.into()),",
                &packet.name, &packet.opcode
            ));
        }

        block
    });

    let server_enum_massive = server_enum_impl
        .new_fn("is_massive")
        .vis("pub")
        .arg_ref_self()
        .ret("bool");

    let matches = server_packets
        .iter()
        .filter_map(|packet| match &packet.mode {
            Some(mode) if mode == "massive" => Some(format!("Self::{}(_)", &packet.name)),
            _ => None,
        })
        .collect::<Vec<String>>()
        .join(" | ");

    server_enum_massive.line(&format!("matches!(self, {})", matches));

    let server_enum_encrypted = server_enum_impl
        .new_fn("is_encrypted")
        .vis("pub")
        .arg_ref_self()
        .ret("bool");

    let matches = server_packets
        .iter()
        .filter_map(|packet| match &packet.mode {
            Some(mode) if mode == "encrypted" => Some(format!("Self::{}(_)", &packet.name)),
            _ => None,
        })
        .collect::<Vec<String>>()
        .join(" | ");

    server_enum_encrypted.line(&format!("matches!(self, {})", matches));

    fs::write(output_file, scope.to_string()).unwrap();
}

fn generate_packets_from<P: AsRef<Path>, S: AsRef<Path>>(
    source_file: P,
    output_path: S,
) -> (String, Vec<PacketReference>, Vec<PacketReference>) {
    let reader = Reader::from_file(source_file).unwrap();
    let collection: PacketCollection = quick_xml::de::from_reader(reader.into_underlying_reader()).unwrap();

    let module_name = collection.module.clone();
    let output_file = output_path.as_ref().join(format!("{}.rs", module_name));
    let mut output_buffer = String::new();
    output_buffer.push_str(PACKET_FILE_HEADER);

    let mut scope = codegen::Scope::new();

    let mut enums = Vec::new();
    let mut structs = Vec::new();
    let mut packets = Vec::new();

    for content in collection.content {
        match content {
            ContentEntry::Enum(enum_def) => enums.push(enum_def),
            ContentEntry::Packet(packet) => packets.push(packet),
            ContentEntry::Struct(struct_def) => structs.push(struct_def),
        }
    }

    for enum_def in enums.iter() {
        structure::generate_enum_def(enum_def, &mut scope);
    }

    for struct_def in structs.iter() {
        structure::generate_helper_struct(struct_def, &mut scope);
    }

    let context = Context {
        structs,
        enums,
        reader_name: READER_VAR_NAME,
        buffer_name: BUFFER_VAR_NAME,
    };
    for packet in packets.iter() {
        structure::generate_struct_for_packet(packet, &context, &mut scope);
    }

    output_buffer.push_str(&scope.to_string());
    fs::write(output_file, output_buffer).unwrap();
    let mut client_packets = Vec::new();
    let mut server_packets = Vec::new();

    for packet in packets.iter() {
        match packet.source {
            PacketSource::Client => client_packets.push(packet.into()),
            PacketSource::Server => server_packets.push(packet.into()),
            PacketSource::Both => {
                client_packets.push(packet.into());
                server_packets.push(packet.into());
            },
        }
    }

    (collection.module.clone(), client_packets, server_packets)
}
