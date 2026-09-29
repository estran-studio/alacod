use crate::fixed_math;
use bevy::prelude::Resource;
use std::collections::BTreeMap;

pub type UUID = String;

#[derive(Debug, Resource, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RollbackRng {
    pub seed: u32,
}

/// Graine de run (hors rollback) : dérivée de la graine de carte, stable pour la session.
#[derive(Debug, Resource, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RunSeed(pub u32);

/// Flux nommés de RNG (ressource rollback) : `BTreeMap<String, RollbackRng>`.
/// Les flux sont dérivés à la demande à partir de la graine de run et du nom du flux.
/// La graine d'un flux = `fnv1a(name) as u32 ^ run_seed`.
#[derive(Debug, Resource, Clone, PartialEq, Eq, Hash)]
pub struct RngStreams {
    pub streams: BTreeMap<String, RollbackRng>,
    pub run_seed: u32,
}

impl RngStreams {
    /// Crée une nouvelle ressource `RngStreams` à partir de la graine de run.
    pub fn new(run_seed: u32) -> Self {
        RngStreams {
            streams: BTreeMap::new(),
            run_seed,
        }
    }

    /// Récupère ou crée un flux nommé. La graine est dérivée de manière déterministe
    /// à partir du nom et de la graine de run via FNV-1a.
    pub fn get_mut(&mut self, name: &str) -> &mut RollbackRng {
        let run_seed = self.run_seed;
        self.streams.entry(name.to_string()).or_insert_with(|| {
            let name_hash = fnv1a(name.as_bytes()) as u32;
            let seed = name_hash ^ run_seed;
            RollbackRng::new(seed)
        })
    }

    /// Variante immutable pour la lecture sans modification.
    pub fn get(&self, name: &str) -> Option<&RollbackRng> {
        self.streams.get(name)
    }
}

impl Default for RngStreams {
    fn default() -> Self {
        RngStreams::new(12345)
    }
}

/// Hash FNV-1a 64 bits, copié de `crates/utils/src/rollback.rs` pour éviter une dépendance.
/// Stable entre les runs et les machines.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
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

    #[test]
    fn test_fnv1a_stability() {
        let hash1 = fnv1a(b"waves");
        let hash2 = fnv1a(b"waves");
        assert_eq!(
            hash1, hash2,
            "FNV-1a should produce identical hashes for the same input"
        );

        let different = fnv1a(b"weapons");
        assert_ne!(
            hash1, different,
            "FNV-1a should produce different hashes for different inputs"
        );
    }

    #[test]
    fn test_rng_streams_independence() {
        let run_seed = 999;
        let mut streams = RngStreams::new(run_seed);

        // Get two different streams
        let stream1 = streams.get_mut("waves");
        let val1_a = stream1.next_u32();
        let val1_b = stream1.next_u32();

        let stream2 = streams.get_mut("weapons");
        let val2_a = stream2.next_u32();
        let val2_b = stream2.next_u32();

        // Streams should produce different values (seeds derived from different names)
        assert_ne!(
            val1_a, val2_a,
            "Different streams should produce different sequences"
        );
        assert_ne!(
            val1_b, val2_b,
            "Different streams should produce different sequences"
        );

        // Advancing one stream should not affect the other
        let stream1_again = streams.get_mut("waves");
        let val1_c = stream1_again.next_u32();
        assert_ne!(
            val1_c, val2_a,
            "Continuing one stream should not affect the other"
        );
    }

    #[test]
    fn test_rng_streams_same_name_same_seed() {
        let run_seed = 888;
        let mut streams1 = RngStreams::new(run_seed);
        let mut streams2 = RngStreams::new(run_seed);

        let seq1: Vec<u32> = (0..5)
            .map(|_| streams1.get_mut("test").next_u32())
            .collect();
        let seq2: Vec<u32> = (0..5)
            .map(|_| streams2.get_mut("test").next_u32())
            .collect();

        assert_eq!(
            seq1, seq2,
            "Same name and run seed should produce identical sequences"
        );
    }

    #[test]
    fn test_rng_streams_order_independence() {
        let run_seed = 777;
        let mut streams1 = RngStreams::new(run_seed);
        let mut streams2 = RngStreams::new(run_seed);

        // Create streams in different orders
        let seq1_waves: Vec<u32> = (0..3)
            .map(|_| streams1.get_mut("waves").next_u32())
            .collect();
        let seq1_weapons: Vec<u32> = (0..3)
            .map(|_| streams1.get_mut("weapons").next_u32())
            .collect();

        let seq2_weapons: Vec<u32> = (0..3)
            .map(|_| streams2.get_mut("weapons").next_u32())
            .collect();
        let seq2_waves: Vec<u32> = (0..3)
            .map(|_| streams2.get_mut("waves").next_u32())
            .collect();

        assert_eq!(
            seq1_waves, seq2_waves,
            "Stream order of creation should not affect sequences"
        );
        assert_eq!(
            seq1_weapons, seq2_weapons,
            "Stream order of creation should not affect sequences"
        );
    }
}
