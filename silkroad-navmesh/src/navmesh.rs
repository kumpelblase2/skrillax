use crate::heightmap::Heightmap;
use crate::Region;
use sr_formats::jmxvnvm::JmxNvm;

pub struct NavmeshContainer {
    region: Region,
    heightmap: Heightmap,
    // objects: Vec<Rc<ObjectMesh>>,
}

impl NavmeshContainer {
    // pub fn build_mesh(&self) -> NavMesh {}

    pub fn new(region: Region, jmx: JmxNvm) -> Self {
        let heightmap = Heightmap::new(jmx.height_map, 96, 20);
        Self { region, heightmap }
    }

    pub fn heightmap(&self) -> &Heightmap {
        &self.heightmap
    }
}
