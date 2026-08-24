use rand::RngExt;

/// Narrow random source required by loot generation, so unit tests can script
/// exact outcomes instead of depending on a specific `rand` sequence.
pub trait LootRandom {
    /// Uniform integer in `[0, bound)`.
    fn below(&mut self, bound: u64) -> u64;
    /// Returns `true` with the given probability.
    fn probability(&mut self, probability: f64) -> bool;
    /// Uniform integer in `[min, max]`.
    fn range(&mut self, min: u32, max: u32) -> u32;
}

/// Production random source wrapping the system RNG.
pub struct SystemRandom;

impl LootRandom for SystemRandom {
    fn below(&mut self, bound: u64) -> u64 {
        rand::rng().random_range(0..bound)
    }

    fn probability(&mut self, probability: f64) -> bool {
        rand::rng().random_bool(probability)
    }

    fn range(&mut self, min: u32, max: u32) -> u32 {
        rand::rng().random_range(min..=max)
    }
}
