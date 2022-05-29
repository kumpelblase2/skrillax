use crate::definition::{EnumValue, PacketAttribute, PacketDefinition, StructDef};

pub trait StructLike {
    fn attributes(&self) -> &Vec<PacketAttribute>;
    fn name(&self) -> &str;
}

impl StructLike for PacketDefinition {
    fn attributes(&self) -> &Vec<PacketAttribute> {
        &self.attributes
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl StructLike for StructDef {
    fn attributes(&self) -> &Vec<PacketAttribute> {
        &self.attributes
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl StructLike for EnumValue {
    fn attributes(&self) -> &Vec<PacketAttribute> {
        &self.attributes
    }

    fn name(&self) -> &str {
        &self.name
    }
}
