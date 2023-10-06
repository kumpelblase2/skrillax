use crate::map_info_ext::MapInfoExt;
use crate::navmesh::NavmeshContainer;
use crate::object::ObjectLoader;
use crate::{get_path_for_region, FileLoader, GlobalNavmesh, MAP_INFO_FILE};
use silkroad_definitions::Region;
use sr_formats::jmxvmfo::JmxMapInfo;
use sr_formats::jmxvnvm::JmxNvm;
use std::collections::HashMap;
use std::io;
use std::io::ErrorKind;
use std::sync::Arc;

pub struct NavmeshBuilder;

impl NavmeshBuilder {
    pub fn build_from(loader: &dyn FileLoader) -> io::Result<GlobalNavmesh> {
        let objects = ObjectLoader::load_objects(loader)?;
        let (_, region_info) = JmxMapInfo::parse(&loader.load_file(MAP_INFO_FILE)?)
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Could not parse map info file."))?;
        let regions = region_info
            .enabled_regions()
            .filter_map(|region| {
                let new_mesh = match Self::load_mesh_for_region(loader, region) {
                    Ok(mesh) => mesh,
                    Err(_) => return None,
                };

                let container = NavmeshContainer::new(region, new_mesh);
                let new_mesh = Arc::new(container);
                Some((region, new_mesh))
            })
            .collect::<HashMap<_, _>>();
        Ok(GlobalNavmesh {
            loaded_meshes: regions,
            loaded_objects: objects,
        })
    }

    fn load_mesh_for_region(loader: &dyn FileLoader, region: Region) -> io::Result<JmxNvm> {
        let path = get_path_for_region(region);
        let data = loader.load_file(&path)?;
        let (_, new_mesh) = JmxNvm::parse(&data)
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Could not parse navmesh file."))?;
        Ok(new_mesh)
    }
}
