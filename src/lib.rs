use rand::{random, RngCore, rngs::SmallRng, SeedableRng};
use zerocopy::AsBytes;

pub use fsc::FixedSizeChunking;
pub use rabin_cdc::RabinCDC;

mod fsc;
mod rabin_cdc;

/// Smallest acceptable value for the minimum chunk size.
const MIN_MIN_CHUNK_SIZE: usize = 64;

/// Largest acceptable value for the minimum chunk size.
const MAX_MIN_CHUNK_SIZE: usize = 1_048_576;

/// Smallest acceptable value for the average chunk size.
const MIN_AVG_CHUNK_SIZE: usize = 256;

/// Largest acceptable value for the average chunk size.
const MAX_AVG_CHUNK_SIZE: usize = 4_194_304;

/// Smallest acceptable value for the average chunk size.
const MIN_MAX_CHUNK_SIZE: usize = 1024;

/// Largest acceptable value for the average chunk size.
const MAX_MAX_CHUNK_SIZE: usize = 16_777_216;

/// Represents parameters for determining chunk sizes.
#[derive(Copy, Clone)]
pub struct ChunkSizeParms {
    /// Minimum expected chunk size.
    pub min_chunk_size: usize,
    /// Average expected chunk size.
    pub avg_chunk_size: usize,
    /// Maximum expected chunk size.
    pub max_chunk_size: usize,
}

/// Represents the chunk structure for the all chunking algorithms.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Chunk {
    /// Starting byte position within the source.
    pub offset: usize,
    /// Length of the chunk in bytes.
    pub length: usize,
}

/// Generates a data block of the specified size using the given seed value.
///
/// # Arguments
///
/// * `size` - The size of the data block to generate.
/// * `seed_value` - The seed value to use for generating the data block.
///
/// # Returns
///
/// A vector of bytes representing the generated data block.
pub fn generate_data_block(size: usize, seed_rnd: Option<u128>) -> Vec<u8> {
    let seed_value = seed_rnd.unwrap_or_else(random);

    let mut seed = [0u8; 32];
    seed.copy_from_slice(seed_value.as_bytes().repeat(2).as_ref());

    let mut rng = SmallRng::from_seed(seed);
    let mut block = vec![0u8; size];
    rng.fill_bytes(&mut block);

    block
}
