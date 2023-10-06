use crate::navmesh::NavmeshContainer;
use crate::object::Object;
use silkroad_definitions::Region;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::{fs, io};

pub mod builder;
pub mod heightmap;
pub mod map_info_ext;
pub mod navmesh;
pub mod object;
pub mod object_info;
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

fn get_path_for_region(region: Region) -> String {
    format!("navmesh/nv_{:04x}.nvm", region.id())
}

const MAP_INFO_FILE: &'static str = "navmesh/mapinfo.mfo";

pub struct GlobalNavmesh {
    loaded_meshes: HashMap<Region, Arc<NavmeshContainer>>,
    #[allow(unused)] // We will eventually use objects for navigation
    loaded_objects: HashMap<u32, Arc<Object>>,
}

impl GlobalNavmesh {
    pub fn mesh_for(&self, region: Region) -> Option<Arc<NavmeshContainer>> {
        self.loaded_meshes.get(&region).cloned()
    }

    pub fn mesh_ref_for(&self, region: Region) -> Option<&NavmeshContainer> {
        self.loaded_meshes.get(&region).map(|arc| arc.as_ref())
    }
}
