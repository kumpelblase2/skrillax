use crate::region::Region;
use sr_formats::jmxvmfo::JmxMapInfo;

pub struct EnabledRegions<'a> {
    region_data: &'a [u8],
    current_region: u16,
}

impl EnabledRegions<'_> {
    fn is_enabled(&self, region: Region) -> bool {
        let region: u16 = region.id();
        let region = region as usize;
        self.region_data[region >> 3] & (128 >> (region % 8)) != 0
    }
}

impl Iterator for EnabledRegions<'_> {
    type Item = Region;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.current_region = match self.current_region.checked_add(1) {
                Some(next_region) => next_region,
                None => return None,
            };

            let region = self.current_region.into();
            if self.is_enabled(region) {
                return Some(region);
            }
        }
    }
}

pub trait MapInfoExt {
    fn enabled_regions(&self) -> EnabledRegions;
}

impl MapInfoExt for JmxMapInfo {
    fn enabled_regions(&self) -> EnabledRegions {
        EnabledRegions {
            region_data: &self.region_data,
            current_region: 0,
        }
    }
}
