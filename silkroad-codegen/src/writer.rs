use crate::code_container::CodeContainer;
use crate::context::Context;
use crate::definition::{EnumDef, PacketAttribute};
use crate::stuct_like::StructLike;

fn generate_writer_for_any<T: CodeContainer>(
    type_name: &str,
    context: &Context,
    container: &mut T,
    accessor: &str,
    with_deref: bool,
) {
    let deref = if with_deref { "*" } else { "" };
    match type_name {
        "u8" => {
            container.new_line(format!("{}.put_u8({}{});", &context.buffer_name, deref, accessor));
        },
        "u16" | "u32" | "u64" | "f32" | "f64" => {
            container.new_line(format!(
                "{}.put_{}_le({}{});",
                &context.buffer_name, type_name, deref, accessor
            ));
        },
        "bool" => {
            container.new_line(format!("{}.put_u8({}{} as u8);", &context.buffer_name, deref, accessor));
        },
        "String" => {
            container
                .new_line(format!(
                    "{}.put_u16_le({}.len() as u16);",
                    &context.buffer_name, accessor
                ))
                .new_line(format!("{}.put_slice({}.as_bytes());", &context.buffer_name, accessor));
        },
        "raw" => {
            container.new_line(format!("{}.put_slice(&{});", &context.buffer_name, accessor));
        },
        "DateTime" => {
            container
                .new_line(format!(
                    "{}.put_u16_le({}.year() as u16);",
                    &context.buffer_name, accessor
                ))
                .new_line(format!(
                    "{}.put_u16_le({}.month() as u16);",
                    &context.buffer_name, accessor
                ))
                .new_line(format!(
                    "{}.put_u16_le({}.day() as u16);",
                    &context.buffer_name, accessor
                ))
                .new_line(format!(
                    "{}.put_u16_le({}.hour() as u16);",
                    &context.buffer_name, accessor
                ))
                .new_line(format!(
                    "{}.put_u16_le({}.minute() as u16);",
                    &context.buffer_name, accessor
                ))
                .new_line(format!(
                    "{}.put_u16_le({}.second() as u16);",
                    &context.buffer_name, accessor
                ))
                .new_line(format!(
                    "{}.put_u32_le({}.timestamp_millis() as u32);",
                    &context.buffer_name, accessor
                ));
        },
        typ => {
            if let Some(struct_def) = context.structs.iter().find(|def| def.name == typ) {
                generate_writer_for_struct(struct_def, context, container, Some(accessor), false);
            } else if let Some(enum_def) = context.enums.iter().find(|def| def.name == typ) {
                generate_writer_for_enum(enum_def, context, container, accessor);
            } else {
                panic!("Unknown unsupported attribute type: {}", typ);
            }
        },
    };
}

pub fn generate_writer_for_struct<S: StructLike, T: CodeContainer>(
    struc: &S,
    context: &Context,
    container: &mut T,
    path: Option<&str>,
    with_deref: bool,
) {
    for attribute in struc.attributes().iter() {
        generate_writer_for(attribute, context, container, path, with_deref);
    }
}

pub fn generate_writer_for_enum<T: CodeContainer>(
    enum_def: &EnumDef,
    context: &Context,
    container: &mut T,
    path: &str,
) {
    container.attach_block(|| {
        let mut block = codegen::Block::new(&format!("match &{}", path));

        for enum_variation in enum_def.values.iter() {
            let writer = enum_def.primitive_type.as_ref().map(|enum_type| {
                let value = enum_variation.value.as_ref().expect("Need value when writing enum");
                match enum_type.as_str() {
                    "u8" => format!("{}.put_u8({})", &context.buffer_name, value),
                    "u16" | "u32" | "u64" | "f32" | "f64" => {
                        format!("{}.put_{}_le({})", &context.buffer_name, enum_type, value)
                    },
                    non_primitive => panic!("Unknown primitive {}", non_primitive),
                }
            });
            if !enum_variation.attributes.is_empty() {
                let all_attributes = enum_variation
                    .attributes
                    .iter()
                    .fold(String::new(), |old, attribute| old + &attribute.name + ", ");

                block.attach_block(|| {
                    let mut inner = codegen::Block::new(&format!(
                        "{}::{} {{ {} }} =>",
                        &enum_def.name, &enum_variation.name, all_attributes
                    ));

                    if let Some(writer) = writer {
                        inner.new_line(format!("{};", writer));
                    }

                    generate_writer_for_struct(enum_variation, context, &mut inner, None, true);

                    inner
                });
            } else {
                block.new_line(format!(
                    "{}::{} => {},",
                    &enum_def.name,
                    &enum_variation.name,
                    writer.unwrap_or_else(|| "{}".to_string())
                ));
            }
        }

        block
    });
}

pub fn generate_writer_for<T: CodeContainer>(
    attribute: &PacketAttribute,
    context: &Context,
    function: &mut T,
    path_string: Option<&str>,
    with_deref: bool,
) {
    let accessor_str = if let Some(path) = path_string {
        format!("{}.{}", path, &attribute.name)
    } else {
        attribute.name.clone()
    };
    let accessor = &accessor_str;

    match attribute.data_type.as_str() {
        "Vec" => {
            let inner = attribute.inner.as_ref().expect("Need inner type for vectors.");
            let list_type = attribute
                .length_type
                .as_ref()
                .cloned()
                .unwrap_or_else(|| "list".to_string());
            match list_type.as_str() {
                "length" => {
                    let length_size = attribute.size.unwrap_or(1);
                    if length_size == 1 {
                        function.new_line(format!("{}.put_u8({}.len() as u8);", &context.buffer_name, accessor));
                    } else {
                        function.new_line(format!(
                            "{}.put_u16_le({}.len() as u16);",
                            &context.buffer_name, accessor
                        ));
                    }
                    function.attach_block(|| {
                        let mut block = codegen::Block::new(&format!("for element in {}.iter()", accessor));

                        generate_writer_for_any(inner, context, &mut block, "element", true);

                        block
                    });
                },
                "has-more" => {
                    function.attach_block(|| {
                        let mut block = codegen::Block::new(&format!("for element in {}.iter()", accessor));

                        block.new_line(format!("{}.put_u8(1);", &context.buffer_name));

                        generate_writer_for_any(inner, context, &mut block, "element", true);

                        block
                    });

                    function.new_line(format!("{}.put_u8(0);", &context.buffer_name));
                },
                "break" => {
                    function.attach_block(|| {
                        let mut block = codegen::Block::new(&format!("for element in {}.iter()", accessor));

                        block.new_line(format!("{}.put_u8(1);", &context.buffer_name));

                        generate_writer_for_any(inner, context, &mut block, "element", true);

                        block
                    });

                    function.new_line(format!("{}.put_u8(2);", &context.buffer_name));
                },
                "none" => {
                    function.attach_block(|| {
                        let mut block = codegen::Block::new(&format!("for element in {}.iter()", accessor));

                        generate_writer_for_any(inner, context, &mut block, "element", true);

                        block
                    });
                },
                s => panic!("unknown list type {}", s),
            }
        },
        "String" => {
            let typ = attribute.size.unwrap_or(1);
            match typ {
                1 => {
                    function
                        .new_line(format!(
                            "{}.put_u16_le({}.len() as u16);",
                            &context.buffer_name, accessor
                        ))
                        .new_line(format!("{}.put_slice({}.as_bytes());", &context.buffer_name, accessor));
                },
                2 => {
                    function.new_line(format!(
                        "{}.put_u16_le({}.len() as u16);",
                        &context.buffer_name, accessor
                    ));

                    function.attach_block(|| {
                        let mut block = codegen::Block::new(&format!("for utf_char in {}.encode_utf16()", accessor));
                        block.new_line(format!("{}.put_u16_le(utf_char);", &context.buffer_name));
                        block
                    });
                },
                size => panic!("Unsupported size for string: {}", size),
            }
        },
        "Option" => {
            let inner = attribute.inner.as_ref().expect("Option needs inner type to work.");
            let length = attribute.length.as_ref().map(|a| *a).unwrap_or(1);

            let func = function.attach_block(|| {
                let mut block = codegen::Block::new(&format!("if let Some({}) = &{}", &attribute.name, accessor));

                if length >= 1 {
                    block.line(format!("{}.put_u8(1);", &context.buffer_name));
                }
                generate_writer_for_any(inner, context, &mut block, &attribute.name, true);

                block
            });
            if length >= 1 {
                func.attach_block(|| {
                    let mut block = codegen::Block::new("else");

                    block.new_line(format!("{}.put_u8(0);", &context.buffer_name));

                    block
                });
            }
        },
        typ => {
            generate_writer_for_any(typ, context, function, accessor, with_deref);
        },
    };
}
