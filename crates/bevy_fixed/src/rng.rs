use crate::fixed_math;
use bevy::prelude::Resource;

pub type UUID = String;

#[derive(Debug, Resource, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RollbackRng {
    pub seed: u32,
}

impl RollbackRng {
    // Constants for the LCG algorithm. These are common choices.
    const A: u32 = 1664525; // Multiplier
    const C: u32 = 1013904223; // Increment
                               // Modulus M is implicitly 2^32 because we are using u32 and letting overflow happen.

    /// Creates a new RNG instance with a given seed.
    /// This seed should ideally be synchronized across all players at the start of the game.
    pub fn new(initial_seed: u32) -> Self {
        RollbackRng { seed: initial_seed }
    }

    /// Generates the next u32 random number.
    /// This method advances the RNG state.
    pub fn next_u32(&mut self) -> u32 {
        // LCG formula: X_n+1 = (a * X_n + c) mod m
        // Here, we use wrapping arithmetic for `mod 2^32`.
        self.seed = self.seed.wrapping_mul(Self::A).wrapping_add(Self::C);
        self.seed
    }

    /// Generates a random Fixed value between 0 (inclusive) and 1 (exclusive).
    /// The type Fixed is assumed to be FixedI32<U16>.
    pub fn next_fixed(&mut self) -> fixed_math::Fixed {
        // Get a raw u32 random number.
        let random_u32 = self.next_u32();

        // To map a u32 value (0 to 2^32-1) to a FixedI32<U16> value in [0, 1):
        // A FixedI32<U16> has 16 fractional bits.
        // The desired value is conceptually (random_u32 / 2^32).
        // To get the raw bits for FixedI32<U16>, we scale this by 2^16:
        // (random_u32 / 2^32) * 2^16 = random_u32 / 2^(32-16) = random_u32 >> 16.
        // The result of `random_u32 >> 16` is a 16-bit integer.
        // `Fixed::from_bits` interprets this integer as the raw representation
        // of the fixed-point number. Since random_u32 is non-negative,
        // the cast to i32 is safe and represents values from 0 up to (2^16-1)/2^16.
        fixed_math::Fixed::from_bits((random_u32 >> 16) as i32)
    }

    /// Generates a random Fixed value between -1 (inclusive) and 1 (exclusive).
    /// The type Fixed is assumed to be FixedI32<U16>.
    pub fn next_fixed_symmetric(&mut self) -> fixed_math::Fixed {
        // Generate a random number in [0, 1)
        let val_0_to_1 = self.next_fixed();

        // Transform to [-1, 1): (val * 2) - 1
        // Fixed::from_num(2) creates the fixed-point representation of 2.
        // Fixed::from_num(1) creates the fixed-point representation of 1.
        (val_0_to_1 * fixed_math::Fixed::from_num(2)) - fixed_math::Fixed::from_num(1)
    }

    /// Generates a random u32 value in the range [min, max).
    /// Note: max is exclusive, min is inclusive.
    pub fn next_u32_range(&mut self, min: u32, max: u32) -> u32 {
        debug_assert!(min < max, "min must be less than max");

        let range = max - min;
        if range == 0 {
            return min;
        }

        // Simple modulo approach
        min + (self.next_u32() % range)
    }

    /// Generates a random u32 value in the range [min, max] (both inclusive).
    pub fn next_u32_range_inclusive(&mut self, min: u32, max: u32) -> u32 {
        debug_assert!(min <= max, "min must be less than or equal to max");

        if min == max {
            return min;
        }

        // Add 1 to make max inclusive
        let range = max - min + 1;
        min + (self.next_u32() % range)
    }

    pub fn next_uuid(&mut self) -> String {
        format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            self.next_u32(),
            self.next_u32() & 0xFFFF,
            self.next_u32() & 0xFFFF,
            self.next_u32() & 0xFFFF,
            ((self.next_u32() as u64) << 32) | (self.next_u32() as u64)
        )
    }

    pub fn next_i32_range(&mut self, min: i32, max: i32) -> i32 {
        debug_assert!(min < max, "min must be less than max");

        // Calculate range as u64 to avoid overflow
        let range = (max as i64 - min as i64) as u64;
        if range == 0 {
            return min;
        }

        // Use modulo to get value in range, then add to min
        min + (self.next_u32() as u64 % range) as i32
    }

    /// Generates a random i32 value in the range [min, max] (both inclusive).
    pub fn next_i32_range_inclusive(&mut self, min: i32, max: i32) -> i32 {
        debug_assert!(min <= max, "min must be less than or equal to max");

        if min == max {
            return min;
        }

        // Calculate range as u64 to avoid overflow, add 1 for inclusive
        let range = (max as i64 - min as i64 + 1) as u64;
        min + (self.next_u32() as u64 % range) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import items from the parent module (RollbackRng)

    #[test]
    fn test_rng_new() {
        let rng = RollbackRng::new(42);
        assert_eq!(rng.seed, 42, "RNG seed should be initialized correctly.");
    }

    #[test]
    fn test_rng_determinism_u32() {
        let mut rng1 = RollbackRng::new(12345);
        let mut rng2 = RollbackRng::new(12345);

        let mut sequence1 = Vec::new();
        let mut sequence2 = Vec::new();

        for _ in 0..100 {
            sequence1.push(rng1.next_u32());
            sequence2.push(rng2.next_u32());
        }

        assert_eq!(
            sequence1, sequence2,
            "Two RNGs with the same seed should produce the same sequence of u32s."
        );
        assert_ne!(rng1.seed, 12345, "RNG seed should change after generation.");
    }

    #[test]
    fn test_rng_determinism_f32() {
        let mut rng1 = RollbackRng::new(54321);
        let mut rng2 = RollbackRng::new(54321);

        let mut sequence1 = Vec::new();
        let mut sequence2 = Vec::new();

        for _ in 0..100 {
            // Pushing f32 directly can have precision issues with assert_eq! on Vecs.
            // For testing determinism, comparing the bit patterns of f32s is more robust if needed,
            // but direct comparison should work for this LCG.
            sequence1.push(rng1.next_fixed());
            sequence2.push(rng2.next_fixed());
        }
        assert_eq!(
            sequence1, sequence2,
            "Two RNGs with the same seed should produce the same sequence of f32s."
        );
    }

    #[test]
    fn test_rng_f32_range() {
        let mut rng = RollbackRng::new(98765);
        for _ in 0..1000 {
            let val = rng.next_fixed();
            assert!(
                (0.0..1.0).contains(&val),
                "next_f32() output {} was not in range [0.0, 1.0)",
                val
            );
        }
    }

    #[test]
    fn test_rng_f32_symmetric_range() {
        let mut rng = RollbackRng::new(112233);
        for _ in 0..1000 {
            let val = rng.next_fixed_symmetric();
            assert!(
                (-1.0..1.0).contains(&val),
                "next_f32_symmetric() output {} was not in range [-1.0, 1.0)",
                val
            );
        }
    }

    #[test]
    fn test_rng_different_seeds_produce_different_sequences() {
        let mut rng1 = RollbackRng::new(100);
        let mut rng2 = RollbackRng::new(200); // Different seed

        let val1 = rng1.next_u32();
        let val2 = rng2.next_u32();

        assert_ne!(
            val1, val2,
            "RNGs with different seeds should produce different first values (highly likely)."
        );

        // Further check a short sequence
        let mut seq1 = vec![val1];
        let mut seq2 = vec![val2];
        for _ in 0..10 {
            seq1.push(rng1.next_u32());
            seq2.push(rng2.next_u32());
        }
        assert_ne!(
            seq1, seq2,
            "RNGs with different seeds should produce different sequences."
        );
    }

    #[test]
    fn test_rng_state_changes() {
        let mut rng = RollbackRng::new(777);
        let initial_seed = rng.seed;
        rng.next_u32();
        assert_ne!(
            rng.seed, initial_seed,
            "Seed should change after calling next_u32."
        );
        let seed_after_u32 = rng.seed;
        rng.next_fixed();
        assert_ne!(
            rng.seed, seed_after_u32,
            "Seed should change after calling next_f32."
        );
        let seed_after_f32 = rng.seed;
        rng.next_fixed_symmetric();
        assert_ne!(
            rng.seed, seed_after_f32,
            "Seed should change after calling next_f32_symmetric."
        );
    }
}
