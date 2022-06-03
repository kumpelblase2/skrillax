use bytes::Bytes;
use std::mem::size_of;

pub trait Size {
    fn calculate_size(&self) -> usize;
}

impl Size for u8 {
    fn calculate_size(&self) -> usize {
        size_of::<u8>()
    }
}

impl Size for u16 {
    fn calculate_size(&self) -> usize {
        size_of::<u16>()
    }
}

impl Size for u32 {
    fn calculate_size(&self) -> usize {
        size_of::<u32>()
    }
}

impl Size for f32 {
    fn calculate_size(&self) -> usize {
        size_of::<f32>()
    }
}

impl Size for u64 {
    fn calculate_size(&self) -> usize {
        size_of::<u64>()
    }
}

impl Size for String {
    fn calculate_size(&self) -> usize {
        2 + self.len()
    }
}

impl Size for bool {
    fn calculate_size(&self) -> usize {
        1
    }
}

impl<T: Size> Size for Option<T> {
    fn calculate_size(&self) -> usize {
        1 + match &self {
            Some(inner) => inner.calculate_size(),
            None => 0,
        }
    }
}

impl Size for Bytes {
    fn calculate_size(&self) -> usize {
        self.len()
    }
}
