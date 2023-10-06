use crate::object_info::ObjectInfo;
use crate::FileLoader;
use log::debug;
use sr_formats::jmxvbms::JmxBMesh;
use sr_formats::jmxvbsr::JmxRes;
use sr_formats::jmxvcpd::{JmxCompound, JmxCompoundHeader};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;

pub enum ObjectFile<'a> {
    Compound(&'a str),
    Resource(&'a str),
    Mesh(&'a str),
}

impl ObjectFile<'_> {
    pub fn from(file_name: &str) -> ObjectFile {
        if file_name.ends_with(".cpd") {
            Ok(ObjectFile::Compound(file_name))
        } else if file_name.ends_with(".bsr") {
            Ok(ObjectFile::Resource(file_name))
        } else if file_name.ends_with(".bms") {
            Ok(ObjectFile::Mesh(file_name))
        } else {
            Err(())
        }
        .expect("Object should end with either of: .cpd, .bsr, or .bms")
    }

    pub fn file_name(&self) -> &str {
        match &self {
            ObjectFile::Compound(name) => name,
            ObjectFile::Resource(name) => name,
            ObjectFile::Mesh(name) => name,
        }
    }
}

pub enum Object {
    Compound {
        header: JmxCompoundHeader,
        collision_resource: Option<JmxRes>,
        resources: Box<[JmxRes]>,
    },
    Resource(JmxRes),
    Mesh(JmxBMesh),
}

impl Object {
    pub fn from(file: &ObjectFile, loader: &dyn FileLoader) -> io::Result<Object> {
        let res = match file {
            ObjectFile::Compound(path) => {
                let data = loader.load_file(path)?;
                let (_, compound) = JmxCompound::parse(&data).expect("Should be able to parse compound data.");
                let collision_resource = compound
                    .collision_resource_path
                    .to_str()
                    .filter(|name| !name.is_empty())
                    .and_then(|path| {
                        let resource_data = loader.load_file(path).ok()?;
                        let (_, resource) = JmxRes::parse(&resource_data).ok()?;
                        Some(resource)
                    });
                let resources = compound
                    .resource_paths
                    .iter()
                    .filter_map(|resource| resource.to_str().filter(|path| !path.is_empty()))
                    .filter_map(|path| {
                        let resource_data = loader.load_file(path).ok()?;
                        let (_, resource) = JmxRes::parse(&resource_data).ok()?;
                        Some(resource)
                    })
                    .collect::<Box<_>>();
                Object::Compound {
                    header: compound.header,
                    collision_resource,
                    resources,
                }
            },
            ObjectFile::Resource(path) => {
                let data = loader.load_file(path)?;
                let (_, resource) = JmxRes::parse(&data).expect("Should be able to parse resource data.");
                Object::Resource(resource)
            },
            ObjectFile::Mesh(path) => {
                let data = loader.load_file(path)?;
                let (_, mesh) = JmxBMesh::parse(&data).expect("Should be able to parse mesh data.");
                Object::Mesh(mesh)
            },
        };
        Ok(res)
    }

    pub fn name(&self) -> &str {
        match &self {
            Object::Compound { header, .. } => header.name.borrow(),
            Object::Resource(res) => res.header.name.borrow(),
            Object::Mesh(mesh) => mesh.header.name.borrow(),
        }
    }
}

pub struct ObjectLoader;

const OBJECT_INFO_FILE: &str = "navmesh/object.ifo";

impl ObjectLoader {
    pub fn load_objects(loader: &dyn FileLoader) -> io::Result<HashMap<u32, Arc<Object>>> {
        let object_info = ObjectInfo::from(&loader.load_file(OBJECT_INFO_FILE)?)?;

        let mut objects = HashMap::new();

        for (id, entry) in object_info.into_iter() {
            let object = ObjectFile::from(entry.file_name());
            match Object::from(&object, loader) {
                Ok(res) => objects.insert(id, Arc::new(res)),
                Err(e) => {
                    debug!("Could not load object: {}", e);
                    continue;
                },
            };
        }

        Ok(objects)
    }
}
