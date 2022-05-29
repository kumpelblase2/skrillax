use crate::definition::{EnumDef, StructDef};

pub struct Context<'a> {
    pub structs: Vec<StructDef>,
    pub enums: Vec<EnumDef>,
    pub reader_name: &'a str,
    pub buffer_name: &'a str,
}
