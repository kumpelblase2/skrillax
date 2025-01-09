use crate::heightmap::Heightmap;
use crate::Region;
use sr_formats::jmxvnvm::JmxNvm;
use std::fmt::{Debug, Formatter};

const MESH_SIZE: usize = 96;
const MESH_TILE_SIZE: usize = 20;

pub struct NavmeshContainer {
    region: Region,
    mesh: JmxNvm,
}

impl Debug for NavmeshContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Navmesh@{}", self.region)
    }
}

impl NavmeshContainer {
    pub fn new(region: Region, jmx: JmxNvm) -> Self {
        Self { region, mesh: jmx }
    }

    pub fn heightmap(&self) -> Heightmap {
        Heightmap::new(&self.mesh.height_map, MESH_SIZE, MESH_TILE_SIZE)
    }
}
