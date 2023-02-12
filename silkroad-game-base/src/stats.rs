const SCALING: f32 = 1.02;

pub struct Stats {
    str: u16,
    int: u16,
}

impl Stats {
    pub fn new(str: u16, int: u16) -> Self {
        Stats { str, int }
    }

    pub fn strength(&self) -> u16 {
        self.str
    }

    pub fn intelligence(&self) -> u16 {
        self.int
    }

    pub fn new_preallocated(str: u16, int: u16) -> Self {
        Stats { str, int }
    }

    pub fn max_health(&self, level: u8) -> u32 {
        let result = (SCALING.powi((level - 1) as i32)) * (self.str * 10) as f32;
        result as u32
    }

    pub fn max_mana(&self, level: u8) -> u32 {
        let result = (SCALING.powi((level - 1) as i32)) * (self.int * 10) as f32;
        result as u32
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new(20, 20)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hp_mana() {
        let default = Stats::default();
        assert_eq!(200, default.max_health(1));
        assert_eq!(200, default.max_mana(1));
    }
}
