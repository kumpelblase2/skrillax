const SCALING: f32 = 1.02;

pub(crate) struct Stats {
    str: u16,
    int: u16,
}

impl Stats {
    pub fn new() -> Self {
        Stats { str: 20, int: 20 }
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
