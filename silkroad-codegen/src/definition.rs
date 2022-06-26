use serde::Deserialize;

#[derive(Deserialize)]
pub struct PacketCollection {
    pub module: String,
    #[serde(rename = "$value")]
    pub content: Vec<ContentEntry>,
}

#[derive(Deserialize)]
pub enum ContentEntry {
    #[serde(rename = "enum")]
    Enum(EnumDef),
    #[serde(rename = "packet")]
    Packet(PacketDefinition),
    #[serde(rename = "struct")]
    Struct(StructDef),
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct EnumDef {
    pub name: String,
    #[serde(rename = "type")]
    pub primitive_type: Option<String>,
    #[serde(rename = "variation")]
    pub values: Vec<EnumValue>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct EnumValue {
    pub name: String,
    pub value: Option<String>,
    #[serde(rename = "attribute", default)]
    pub attributes: Vec<PacketAttribute>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct StructDef {
    pub name: String,
    #[serde(rename = "attribute", default)]
    pub attributes: Vec<PacketAttribute>,
}

#[derive(Deserialize, Eq, PartialOrd, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PacketSource {
    Client,
    Server,
    Both,
}

impl PacketSource {
    pub fn should_generate_from_client(&self) -> bool {
        match self {
            PacketSource::Client | PacketSource::Both => true,
            PacketSource::Server => false,
        }
    }

    pub fn should_generate_from_server(&self) -> bool {
        match self {
            PacketSource::Server | PacketSource::Both => true,
            PacketSource::Client => false,
        }
    }
}

pub struct PacketReference {
    pub name: String,
    pub opcode: String,
    pub mode: Option<String>,
}

impl From<&PacketDefinition> for PacketReference {
    fn from(def: &PacketDefinition) -> Self {
        PacketReference {
            name: def.name.clone(),
            opcode: def.opcode.clone(),
            mode: def.option.clone(),
        }
    }
}

#[derive(Deserialize)]
pub struct PacketDefinition {
    pub name: String,
    pub opcode: String,
    pub source: PacketSource,
    pub option: Option<String>,
    #[serde(rename = "attribute", default)]
    pub attributes: Vec<PacketAttribute>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct PacketAttribute {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub value: Option<String>,
    pub length: Option<usize>,
    pub inner: Option<String>,
    pub size: Option<usize>,
    #[serde(rename = "if")]
    pub if_condition: Option<String>,
    #[serde(rename = "list-type")]
    pub length_type: Option<String>,
}
