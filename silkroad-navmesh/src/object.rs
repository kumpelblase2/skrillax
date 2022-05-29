use sr_formats::jmxvbms::JmxBMesh;
use sr_formats::jmxvbsr::JmxRes;
use sr_formats::jmxvcpd::JmxCompound;
use std::borrow::Borrow;

pub enum ObjectFile {
    Compound(String),
    Resource(String),
    Mesh(String),
}

impl ObjectFile {
    pub fn from(file_name: String) -> ObjectFile {
        if file_name.ends_with(".cpd") {
            ObjectFile::Compound(file_name)
        } else if file_name.ends_with(".bsr") {
            ObjectFile::Resource(file_name)
        } else if file_name.ends_with(".bms") {
            ObjectFile::Mesh(file_name)
        } else {
            panic!("Unknown file extension for file {}.", file_name)
        }
    }

    pub fn file_name(&self) -> &str {
        match &self {
            ObjectFile::Compound(name) => name.as_str(),
            ObjectFile::Resource(name) => name.as_str(),
            ObjectFile::Mesh(name) => name.as_str(),
        }
    }
}

pub enum Object {
    Compound(JmxCompound),
    Resource(JmxRes),
    Mesh(JmxBMesh),
}

impl Object {
    pub fn load_from(file: &ObjectFile, data: &[u8]) -> Object {
        match file {
            ObjectFile::Compound(_) => {
                let (_, compound) = JmxCompound::parse(data).unwrap();
                Object::Compound(compound)
            },
            ObjectFile::Resource(_) => {
                let (_, resource) = JmxRes::parse(data).unwrap();
                Object::Resource(resource)
            },
            ObjectFile::Mesh(_) => {
                let (_, mesh) = JmxBMesh::parse(data).unwrap();
                Object::Mesh(mesh)
            },
        }
    }

    pub fn name(&self) -> &str {
        match &self {
            Object::Compound(cpd) => cpd.header.name.borrow(),
            Object::Resource(res) => res.header.name.borrow(),
            Object::Mesh(mesh) => mesh.header.name.borrow(),
        }
    }
}
