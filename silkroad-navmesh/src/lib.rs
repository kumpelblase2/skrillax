use crate::navmesh::NavmeshContainer;
use crate::object_mesh::ObjectMesh;
use silkroad_definitions::Region;
use sr_formats::jmxvnvm::JmxNvm;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{fs, io};

mod heightmap;
pub mod map_info_ext;
pub mod navmesh;
pub mod object;
pub mod object_info;
mod object_mesh;
pub mod region;

pub trait FileLoader {
    fn load_file(&self, file_path: &str) -> io::Result<Vec<u8>>;
}

#[cfg(feature = "pk2")]
impl FileLoader for pk2::Pk2 {
    fn load_file(&self, file_path: &str) -> io::Result<Vec<u8>> {
        self.read(format!("/{}", file_path))
    }
}

impl FileLoader for Path {
    fn load_file(&self, file_path: &str) -> io::Result<Vec<u8>> {
        fs::read(self.join(file_path))
    }
}

pub struct NavmeshLoader<T: FileLoader> {
    storage_loader: Arc<Mutex<T>>,
    loaded_meshes: HashMap<Region, Arc<NavmeshContainer>>,
    loaded_objects: HashMap<u32, Arc<ObjectMesh>>,
}

fn get_path_for_region(region: Region) -> String {
    format!("navmesh/nv_{:04x}.nvm", region.id())
}

impl<T: FileLoader> NavmeshLoader<T> {
    pub fn new(loader: T) -> NavmeshLoader<T> {
        NavmeshLoader {
            storage_loader: Arc::new(Mutex::new(loader)),
            loaded_meshes: HashMap::new(),
            loaded_objects: HashMap::new(),
        }
    }

    pub fn load_navmesh(&mut self, region: Region) -> io::Result<Arc<NavmeshContainer>> {
        return match self.loaded_meshes.get(&region) {
            Some(mesh) => Ok(mesh.clone()),
            None => {
                let path = get_path_for_region(region);
                let data = self.storage_loader.lock().unwrap().load_file(&path)?;
                let (_, new_mesh) = JmxNvm::parse(&data).map_err(|_| io::Error::new(ErrorKind::InvalidData, ""))?;
                let container = NavmeshContainer::new(region, new_mesh);
                let new_mesh = Arc::new(container);
                self.loaded_meshes.insert(region, new_mesh.clone());

                Ok(new_mesh)
            },
        };
    }
}
