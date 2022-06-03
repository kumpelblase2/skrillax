pub struct IdAllocator {
    last_id: u32,
}

impl IdAllocator {
    pub fn new() -> Self {
        Self { last_id: 0 }
    }

    pub fn allocate(&mut self) -> u32 {
        self.last_id += 1;
        self.last_id
    }
}
