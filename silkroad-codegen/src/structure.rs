use codegen::{Block, Function, Scope};

use crate::context::Context;
use crate::definition::{EnumDef, PacketAttribute, PacketDefinition, StructDef};
use crate::{reader, writer, CodeContainer, BUFFER_VAR_NAME};

pub fn generate_enum_def(enum_def: &EnumDef, scope: &mut Scope) {
    let enum_ = scope.new_enum(&enum_def.name).vis("pub").derive("Clone");

    let no_struct_variants = enum_def.values.iter().all(|variant| variant.attributes.is_empty());
    if no_struct_variants {
        enum_.derive("PartialEq").derive("PartialOrd").derive("Copy");
    }

    for enum_value in enum_def.values.iter() {
        let variant = enum_.new_variant(&enum_value.name);
        enum_value.attributes.iter().for_each(|attr| {
            let typ = get_full_attribute_type(attr);
            variant.named(&attr.name, typ);
        });
    }

    if !no_struct_variants {
        let enum_impl = scope.new_impl(&enum_def.name);
        for enum_value in enum_def.values.iter() {
            if !enum_value.attributes.is_empty() {
                let lower_name = enum_value.name.to_lowercase();
                let create_fn = enum_impl.new_fn(&lower_name).vis("pub").ret("Self");
                let assignment_string = generate_new_args(&enum_value.attributes, create_fn);

                create_fn.line(format!(
                    "{}::{} {{ {} }}",
                    &enum_def.name, &enum_value.name, assignment_string
                ));
            }
        }
    }

    let size_impl = scope.new_impl(&enum_def.name).impl_trait("Size");
    let size_fn = size_impl.new_fn("calculate_size").arg_ref_self().ret("usize");
    let minimum_size = enum_def
        .primitive_type
        .as_ref()
        .map(|primitiv| format!("std::mem::size_of::<{}>()", primitiv))
        .unwrap_or("0".to_string());

    if no_struct_variants {
        size_fn.line(minimum_size);
    } else {
        size_fn.attach_block(|| {
            let mut block = Block::new(&format!("{} + match &self", minimum_size));
            for enum_variant in enum_def.values.iter() {
                if enum_variant.attributes.is_empty() {
                    block.line(&format!("{}::{} => {},", &enum_def.name, &enum_variant.name, "0"));
                } else {
                    let destructuring = enum_variant
                        .attributes
                        .iter()
                        .map(|attr| attr.name.clone())
                        .collect::<Vec<String>>()
                        .join(", ");
                    block.attach_block(|| {
                        let mut block = Block::new(&format!(
                            "{}::{} {{ {} }} =>",
                            &enum_def.name, &enum_variant.name, destructuring
                        ));
                        let size_calc = enum_variant
                            .attributes
                            .iter()
                            .map(|attr| generate_size_calc_for(attr, false))
                            .collect::<Vec<String>>()
                            .join(" + ");
                        block.line(size_calc);
                        block
                    });
                }
            }
            block
        });
    }
}

fn generate_size_calc_for(attr: &PacketAttribute, in_struct: bool) -> String {
    let name_ref = if in_struct {
        format!("self.{}", &attr.name)
    } else {
        attr.name.clone()
    };

    match attr.data_type.as_str() {
        "Vec" => match attr.length_type.as_ref().expect("Missing inner type for Vec").as_str() {
            "length" => format!(
                "2 + {}.iter().map(|inner| inner.calculate_size()).sum::<usize>()",
                name_ref
            ),
            "has-more" | "break" => format!(
                "{}.iter().map(|inner| inner.calculate_size() + 1).sum::<usize>()",
                name_ref
            ),
            "none" => format!("{}.iter().map(|inner| inner.calculate_size()).sum::<usize>()", name_ref),
            _ => panic!("Unknown length type"),
        },
        "DateTime" => String::from("14"),
        "String" if attr.length.unwrap_or(1) == 2 => {
            format!("2 + {}.len() * 2", name_ref)
        },
        _ => format!("{}.calculate_size()", name_ref),
    }
}

fn generate_new_args(attributes: &[PacketAttribute], create_fn: &mut Function) -> String {
    let mut assignment_string = String::new();
    for attr in attributes.iter() {
        match &attr.value {
            Some(value) => {
                assignment_string.push_str(&attr.name);
                assignment_string.push_str(": ");
                assignment_string.push_str(value);
                assignment_string.push_str(", ")
            },
            None => {
                create_fn.arg(&attr.name, get_full_attribute_type(attr));
                assignment_string.push_str(&attr.name);
                assignment_string.push_str(", ");
            },
        }
    }
    assignment_string
}

pub fn generate_helper_struct(struct_def: &StructDef, scope: &mut Scope) {
    let helper_struct = scope.new_struct(&struct_def.name).vis("pub").derive("Clone");

    for attr in struct_def.attributes.iter() {
        generate_attribute(attr, helper_struct);
    }

    create_new_for(&struct_def.name, &struct_def.attributes, scope);

    generate_size_impl(&struct_def.name, &struct_def.attributes, scope);
}

fn get_full_attribute_type(attr: &PacketAttribute) -> String {
    match attr.data_type.as_str() {
        "Vec" | "Option" => format!(
            "{}<{}>",
            &attr.data_type,
            attr.inner.as_ref().expect("Missing inner type for vec in struct")
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

pub fn generate_struct_for_packet(packet: &PacketDefinition, context: &Context, scope: &mut codegen::Scope) {
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

            reader::generate_reader_for_struct(packet, context, convert_fn, |st| format!("Ok({})", &st));
        }

        let from_impl = scope
            .new_impl("ClientPacket")
            .impl_trait(format!("From<{}>", &packet.name));
        {
            let from_fn = from_impl.new_fn("from").arg("other", &packet.name).ret("Self");
            from_fn.line(format!("ClientPacket::{}(other)", &packet.name));
        }
    }

    if packet.source.should_generate_from_server() {
        let to_impl = scope.new_impl("Bytes").impl_trait(format!("From<{}>", &packet.name));
        {
            let convert_fn = to_impl.new_fn("from").arg("op", &packet.name).ret("Bytes");

            convert_fn.line(format!(
                "let mut {} = BytesMut::with_capacity(op.calculate_size());",
                BUFFER_VAR_NAME
            ));

            writer::generate_writer_for_struct(packet, context, convert_fn, Some("op"), false);

            convert_fn.line(format!("{}.freeze()", BUFFER_VAR_NAME));
        }

        let from_impl = scope
            .new_impl("ServerPacket")
            .impl_trait(format!("From<{}>", &packet.name));
        {
            let from_fn = from_impl.new_fn("from").arg("other", &packet.name).ret("Self");
            from_fn.line(format!("ServerPacket::{}(other)", &packet.name));
        }

        create_new_for(&packet.name, &packet.attributes, scope);

        generate_size_impl(&packet.name, &packet.attributes, scope);
    }
}

fn create_new_for(name: &str, attributes: &[PacketAttribute], scope: &mut Scope) {
    let new_fn = scope.new_impl(name).new_fn("new").ret("Self").vis("pub");
    let assignment_string = generate_new_args(attributes, new_fn);
    new_fn.line(format!("{} {{ {} }}", name, &assignment_string));
}

fn generate_size_impl(name: &str, attributes: &[PacketAttribute], scope: &mut Scope) {
    let size_impl = scope.new_impl(name).impl_trait("Size");
    let size_fn = size_impl.new_fn("calculate_size").arg_ref_self().ret("usize");
    if attributes.is_empty() {
        size_fn.line("0");
    } else {
        let size_calc = attributes
            .iter()
            .map(|attr| generate_size_calc_for(attr, true))
            .collect::<Vec<String>>()
            .join(" + ");
        size_fn.line(size_calc);
    }
}
