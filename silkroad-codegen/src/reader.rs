use codegen::Block;

use crate::code_container::CodeContainer;
use crate::context::Context;
use crate::definition::PacketAttribute;
use crate::stuct_like::StructLike;

pub fn generate_reader_for_struct<S: StructLike, T: CodeContainer, O: FnOnce(String) -> String>(
    struct_def: &S,
    context: &Context,
    container: &mut T,
    output: O,
) {
    for attr in struct_def.attributes().iter() {
        generate_reader_for(attr, context, container);
    }

    let all_assignments = struct_def
        .attributes()
        .iter()
        .fold(String::new(), |old, attribute| old + &attribute.name + ", ");

    let output_string = format!("{} {{ {} }}", struct_def.name(), all_assignments);
    container.new_line(output(output_string));
}

fn generate_reader_for<T: CodeContainer>(attribute: &PacketAttribute, context: &Context, container: &mut T) {
    match attribute.data_type.as_str() {
        "u8" => container.new_line(format!(
            "let {} = {}.read_u8()?;",
            &attribute.name, &context.reader_name
        )),
        "u16" | "u32" | "u64" | "f32" | "f64" => container.new_line(format!(
            "let {} = {}.read_{}::<byteorder::LittleEndian>()?;",
            &attribute.name, &context.reader_name, &attribute.data_type
        )),
        "bool" => container.new_line(format!(
            "let {} = {}.read_u8()? as bool;",
            &attribute.name, &context.reader_name
        )),
        "raw" => {
            let length = attribute.length.as_ref().expect("Missing length for raw attribute.");
            container
                .new_line(format!(
                    "let mut {}_raw = BytesMut::with_capacity({});",
                    &attribute.name, length
                ))
                .attach_block(|| {
                    let mut block = Block::new(&format!("for _ in 0..{}", length));
                    block.new_line(format!(
                        "\t{}_raw.put_u8({}.read_u8()?);",
                        &attribute.name, &context.reader_name
                    ));
                    block
                })
                .new_line(format!("let {} = {}_raw.freeze();", &attribute.name, &attribute.name))
        },
        "String" => {
            container.new_line(format!(
                "let {}_string_len = {}.read_u16::<byteorder::LittleEndian>()?;",
                &attribute.name, &context.reader_name
            ));

            let typ = attribute.size.unwrap_or(1);
            match typ {
                1 => container
                    .new_line(format!(
                        "let mut {}_bytes = Vec::with_capacity({}_string_len as usize);",
                        &attribute.name, &attribute.name
                    ))
                    .attach_block(|| {
                        let mut block = Block::new(&format!("for _ in 0..{}_string_len", &attribute.name));
                        block.new_line(format!(
                            "\t{}_bytes.push({}.read_u8()?);",
                            &attribute.name, &context.reader_name
                        ));
                        block
                    })
                    .new_line(format!(
                        "let {} = String::from_utf8({}_bytes)?;",
                        &attribute.name, &attribute.name
                    )),
                2 => container
                    .new_line(format!(
                        "let mut {}_bytes = Vec::with_capacity({}_string_len as usize);",
                        &attribute.name, &attribute.name
                    ))
                    .attach_block(|| {
                        let mut block = Block::new(&format!("for _ in 0..{}_string_len", &attribute.name));
                        block.new_line(format!(
                            "\t{}_bytes.push({}.read_u16::<byteorder::LittleEndian>()?);",
                            &attribute.name, &context.reader_name
                        ));
                        block
                    })
                    .new_line(format!(
                        "let {} = String::from_utf16(&{}_bytes)?;",
                        &attribute.name, &attribute.name
                    )),
                size => panic!("Unsupported string length: {}", size),
            }
        },
        "Vec" => {
            let inner = attribute.inner.as_ref().expect("Need inner type for vectors.");
            let list_type = attribute
                .length_type
                .as_ref()
                .cloned()
                .unwrap_or_else(|| String::from("list"));
            let struct_def = context
                .structs
                .iter()
                .find(|str_def| &str_def.name == inner)
                .expect("Cannot find helper struct");
            match list_type.as_str() {
                "length" => container
                    .new_line(format!(
                        "let {}_size = {}.read_u8()?;",
                        &attribute.name, &context.reader_name
                    ))
                    .new_line(format!(
                        "let {}:Vec<{}> = Vec::with_capacity({}_size);",
                        &attribute.name, inner, &attribute.name
                    ))
                    .attach_block(|| {
                        let mut block = codegen::Block::new(&format!("for _ in 0..{}_size", &attribute.name));

                        generate_reader_for_struct(struct_def, context, &mut block, |st| {
                            format!("{}.push({})", &attribute.name, &st)
                        });

                        block
                    }),
                "has-more" => container
                    .new_line(format!("let {}:Vec<{}> = Vec::new();", &attribute.name, inner))
                    .attach_block(|| {
                        let mut block = codegen::Block::new(&format!("while {}.read_u8()? == 1", &attribute.name));

                        generate_reader_for_struct(struct_def, context, &mut block, |st| {
                            format!("{}.push({})", &attribute.name, &st)
                        });

                        block
                    }),
                "break" => container
                    .new_line(format!("let {}:Vec<{}> = Vec::new();", &attribute.name, inner))
                    .attach_block(|| {
                        let mut block = codegen::Block::new(&format!("while {}.read_u8()? != 2", &attribute.name));

                        generate_reader_for_struct(struct_def, context, &mut block, |st| {
                            format!("{}.push({})", &attribute.name, &st)
                        });

                        block
                    }),
                s => panic!("unknown list type {}", s),
            }
        },
        "Option" => {
            let inner = attribute.inner.as_ref().expect("Need inner type for options.");
            let struct_def = context
                .structs
                .iter()
                .find(|str_def| &str_def.name == inner)
                .expect("Cannot find helper struct");
            container
                .attach_block(|| {
                    let mut block = codegen::Block::new(&format!(
                        "let {} = if {}.read_u8()? == 1",
                        &attribute.name, &context.reader_name
                    ));

                    generate_reader_for_struct(struct_def, context, &mut block, |st| format!("Some({})", &st));

                    block
                })
                .attach_block(|| {
                    let mut block = codegen::Block::new("else");
                    block.new_line("None");
                    block
                })
        },
        typ => {
            if let Some(struct_def) = context.structs.iter().find(|def| def.name == typ) {
                container.new_line(format!("let {} = {{", &attribute.name));
                container.attach_block(|| {
                    let mut block = codegen::Block::new(&format!("let {} = ", &attribute.name));

                    generate_reader_for_struct(struct_def, context, &mut block, |st| st);

                    block
                })
            } else if let Some(enum_def) = context.enums.iter().find(|def| def.name == typ) {
                let enum_type = enum_def
                    .primitive_type
                    .as_ref()
                    .expect("Need variation value for reading")
                    .as_str();
                let reader = match enum_type {
                    "u8" => "read_u8()?".to_string(),
                    "u16" | "u32" | "u64" => {
                        format!("read_{}::<byteorder::LittleEndian>()?", enum_type)
                    },
                    non_primitive => panic!("Unknown primitive enum type {}", non_primitive),
                };
                container.attach_block(|| {
                    let mut block = codegen::Block::new(&format!(
                        "let {} = match {}.{}",
                        &attribute.name, &context.reader_name, &reader
                    ));
                    block.after(";");

                    for enum_variation in enum_def.values.iter() {
                        let enum_variation_value =
                            enum_variation.value.as_ref().expect("Need variation value for reading");
                        if !enum_variation.attributes.is_empty() {
                            block.attach_block(|| {
                                let mut inner = codegen::Block::new(&format!("{} => ", enum_variation_value));

                                generate_reader_for_struct(enum_variation, context, &mut inner, |st| {
                                    format!("{}::{}", &enum_def.name, &st)
                                });

                                inner
                            });
                        } else {
                            block.new_line(format!(
                                "{} => {}::{},",
                                enum_variation_value, &enum_def.name, &enum_variation.name
                            ));
                        }
                    }

                    // block.new_line(format!("unknown => return Err(anyhow!(\"Unknown {} variation for value {{}}\", unknown))", &enum_def.name));
                    block.new_line(format!(
                        "unknown => return Err(ProtocolError::UnknownVariation(unknown, \"{}\"))",
                        &enum_def.name
                    ));

                    block
                })
            } else {
                panic!("Unknown unsupported attribute type: {}", &attribute.data_type)
            }
        },
    };
}
