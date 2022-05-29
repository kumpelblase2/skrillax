use codegen::Scope;

use crate::context::Context;
use crate::definition::{EnumDef, PacketAttribute, PacketDefinition, StructDef};
use crate::{reader, writer, BUFFER_VAR_NAME};

pub fn generate_enum_def(enum_def: &EnumDef, scope: &mut Scope) {
    let enum_ = scope.new_enum(&enum_def.name).vis("pub").derive("Clone");

    let no_struct_variants = enum_def
        .values
        .iter()
        .all(|variant| variant.attributes.is_empty());
    if no_struct_variants {
        enum_.derive("PartialEq").derive("PartialOrd");
    }

    for enum_value in enum_def.values.iter() {
        let variant = enum_.new_variant(&enum_value.name);
        enum_value.attributes.iter().for_each(|attr| {
            let typ = get_full_attribute_type(attr);
            variant.named(&attr.name, typ);
        });
    }
}

pub fn generate_helper_struct(struct_def: &StructDef, scope: &mut Scope) {
    let helper_struct = scope
        .new_struct(&struct_def.name)
        .vis("pub")
        .derive("Clone");

    for attr in struct_def.attributes.iter() {
        generate_attribute(attr, helper_struct);
    }
}

fn get_full_attribute_type(attr: &PacketAttribute) -> String {
    match attr.data_type.as_str() {
        "Vec" | "Option" => format!(
            "{}<{}>",
            &attr.data_type,
            attr.inner
                .as_ref()
                .expect("Missing inner type for vec in struct")
        ),
        "DateTime" => "DateTime<Utc>".to_string(),
        "raw" => "Bytes".to_string(),
        typ => typ.to_string(),
    }
}

fn generate_attribute(attr: &PacketAttribute, struc: &mut codegen::Struct) {
    let name = format!("pub {}", &attr.name);
    let typ = get_full_attribute_type(attr);
    struc.field(&name, typ);
}

pub fn generate_struct_for_packet(
    packet: &PacketDefinition,
    context: &Context,
    scope: &mut codegen::Scope,
) {
    let packet_struct = scope.new_struct(&packet.name).vis("pub").derive("Clone");
    for attribute in packet.attributes.iter() {
        generate_attribute(attribute, packet_struct);
    }

    if packet.source.should_generate_from_client() {
        let from_impl = scope
            .new_impl(&packet.name)
            .impl_trait("TryFrom<Bytes>")
            .associate_type("Error", "ProtocolError");

        {
            let convert_fn = from_impl
                .new_fn("try_from")
                .arg("data", "Bytes")
                .ret("Result<Self, Self::Error>");

            convert_fn.line(format!("let mut {} = data.reader();", &context.reader_name));

            reader::generate_reader_for_struct(packet, context, convert_fn, |st| {
                format!("Ok({})", &st)
            });
        }

        let into_impl = scope
            .new_impl(&packet.name)
            .impl_trait("Into<ClientPacket>");
        {
            let into_fn = into_impl.new_fn("into").arg_self().ret("ClientPacket");

            into_fn.line(format!("ClientPacket::{}(self)", packet.name));
        }
    }

    if packet.source.should_generate_from_server() {
        let to_impl = scope
            .new_impl("Bytes")
            .impl_trait(format!("From<{}>", &packet.name));
        {
            let convert_fn = to_impl.new_fn("from").arg("op", &packet.name).ret("Bytes");

            convert_fn.line(format!("let mut {} = BytesMut::new();", BUFFER_VAR_NAME));

            writer::generate_writer_for_struct(packet, context, convert_fn, Some("op"), false);

            convert_fn.line(format!("{}.freeze()", BUFFER_VAR_NAME));
        }

        let into_impl = scope
            .new_impl(&packet.name)
            .impl_trait("Into<ServerPacket>");
        {
            let into_fn = into_impl.new_fn("into").arg_self().ret("ServerPacket");

            into_fn.line(format!("ServerPacket::{}(self)", packet.name));
        }

        let new_fn = scope
            .new_impl(&packet.name)
            .new_fn("new")
            .ret("Self")
            .vis("pub");
        let mut assignment_string = String::new();
        for arg in packet.attributes.iter() {
            match &arg.value {
                Some(value) => {
                    assignment_string.push_str(&arg.name);
                    assignment_string.push_str(": ");
                    assignment_string.push_str(value);
                    assignment_string.push_str(", ")
                }
                None => {
                    new_fn.arg(&arg.name, get_full_attribute_type(arg));
                    assignment_string.push_str(&arg.name);
                    assignment_string.push_str(", ");
                }
            }
        }

        new_fn.line(format!("{} {{ {} }}", &packet.name, &assignment_string));
    }
}
