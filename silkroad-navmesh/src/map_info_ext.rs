use crate::region::Region;
use sr_formats::jmxvmfo::JmxMapInfo;

pub struct EnabledRegions<'a> {
    region_data: &'a [u8],
    current_region: u16,
    ended: bool,
}

impl<'a> EnabledRegions<'a> {
    pub fn new(region_data: &'a [u8]) -> EnabledRegions<'a> {
        Self {
            region_data,
            current_region: 0,
            ended: region_data.is_empty(),
        }
    }

    fn is_enabled(&self, region: Region) -> bool {
        let region: u16 = region.id();
        let region = region as usize;
        let index = region >> 3;
        if index >= self.region_data.len() {
            return false;
        }

        self.region_data[index] & (128 >> (region % 8)) != 0
    }

    fn step(&mut self) -> u16 {
        let index = self.current_region;

        self.current_region = match self.current_region.checked_add(1) {
            Some(next_region) => next_region,
            None => {
                self.ended = true;
                index
            },
        };

        index
    }
}

impl Iterator for EnabledRegions<'_> {
    type Item = Region;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.ended {
            let region_index = self.step();
            let region = region_index.into();
            if self.is_enabled(region) {
                return Some(region);
            }
        }
        None
    }
}

pub trait MapInfoExt {
    fn enabled_regions(&self) -> EnabledRegions;
}

impl MapInfoExt for JmxMapInfo {
    fn enabled_regions(&self) -> EnabledRegions {
        EnabledRegions::new(&self.region_data)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_empty_list() {
        let enabled_regions: Vec<u8> = vec![];
        let region_iterator = EnabledRegions::new(&enabled_regions);
        let all = region_iterator.collect::<Vec<_>>();
        assert_eq!(all.len(), 0);
    }

    #[test]
    pub fn test_single_entry() {
        let enabled_regions: Vec<u8> = vec![0b10000000, 0b10000001];
        let region_iterator = EnabledRegions::new(&enabled_regions);
        let all = region_iterator.collect::<Vec<_>>();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&Region::new(0)));
        assert!(all.contains(&Region::new(8)));
        assert!(all.contains(&Region::new(15)));
    }
}
