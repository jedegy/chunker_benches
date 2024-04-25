/// Constants commonly used in Rabin fingerprint calculations.
/// Taken from: https://github.com/moinakg/pcompress
const PRIME: u64 = 153_191u64;
const MASK: u64 = 0x00ff_ffff_ffffu64;

/// Irreducible polynomial for Rabin modulus, used in pcompress.
const FP_POLY: u64 = 0xbfe6_b8a5_bf37_8d83u64;

/// Minimum and maximum window sizes for chunking.
const MIN_WIN_SIZE: usize = 8;
const MAX_WIN_SIZE: usize = 64;

/// Rabin chunker for data segmentation based on Rabin fingerprinting algorithm.
pub struct RabinCDC<'a> {
    /// Precomputed table mapping each output byte to a polynomial value.
    out_map: Vec<u64>,
    /// Precomputed irreducible polynomial table for Rabin fingerprint.
    ir: Vec<u64>,
    /// Bit mask for window indexing, facilitating circular buffer behavior.
    win_mask: usize,
    /// Current position for the sliding window.
    cur_pos: usize,
    /// Mask used to determine chunk cuts.
    cut_mask: u64,
    /// Data buffer to chunk.
    source: &'a [u8],
    /// Parameters specifying minimum, average, and maximum chunk sizes.
    chunk_parms: super::ChunkSizeParms,
}

impl<'a> RabinCDC<'a> {
    /// Constructs a new `RabinCDC`.
    ///
    /// # Arguments
    /// * `source` - Data buffer to be chunked.
    /// * `win_size` - Size of the sliding window for chunk determination
    /// * `min_size` - Minimum chunk size.
    /// * `avg_size` - Average chunk size.
    /// * `max_size` - Maximum chunk size.
    ///
    /// # Panics
    ///
    /// Panic if any of the size constraints are violated or if `win_size` is not a power of two.
    ///
    /// # Returns
    ///
    /// A new `RabinCDC` instance.
    pub fn new(
        source: &'a [u8],
        win_size: usize,
        min_size: usize,
        avg_size: usize,
        max_size: usize,
    ) -> Self {
        // Assertions to ensure the parameters are within expected bounds
        assert!(
            (MIN_WIN_SIZE..=MAX_WIN_SIZE).contains(&win_size),
            "Window size out of valid range"
        );
        assert!(
            win_size & (win_size - 1) == 0 && win_size != 0,
            "Window size must be a power of two"
        );
        assert!(
            (super::MIN_MIN_CHUNK_SIZE..=super::MAX_MIN_CHUNK_SIZE).contains(&min_size),
            "Min chunk size out of valid range"
        );
        assert!(
            (super::MIN_AVG_CHUNK_SIZE..=super::MAX_AVG_CHUNK_SIZE).contains(&avg_size),
            "Average chunk size out of valid range"
        );
        assert!(
            (super::MIN_MAX_CHUNK_SIZE..=super::MAX_MAX_CHUNK_SIZE).contains(&max_size),
            "Max chunk size out of valid range"
        );

        let poly_pow = (1..=win_size).fold(1u64, |acc, _| (acc * PRIME) & MASK);

        let out_map = (0..256)
            .map(|i| (i as u64 * poly_pow) & MASK)
            .collect::<Vec<u64>>();

        let ir = (0..256)
            .map(|_| {
                (0..win_size).fold(1u64, |acc, _| {
                    if acc & FP_POLY != 0 {
                        (acc + (acc * PRIME)) & MASK
                    } else {
                        (acc * PRIME) & MASK
                    }
                })
            })
            .collect::<Vec<u64>>();

        Self {
            out_map,
            ir,
            win_mask: win_size - 1,
            cur_pos: 0,
            cut_mask: (avg_size - min_size - 1) as u64,
            source,
            chunk_parms: super::ChunkSizeParms {
                min_chunk_size: min_size,
                avg_chunk_size: avg_size,
                max_chunk_size: max_size,
            },
        }
    }
}

impl Iterator for RabinCDC<'_> {
    type Item = super::Chunk;

    /// Computes the next chunk based on the Rabin fingerprint.
    ///
    /// # Returns
    ///
    /// Returns a `Chunk` if the conditions for a chunk boundary are met, otherwise `None` if an
    /// end of data is reached.
    fn next(&mut self) -> Option<Self::Item> {
        if self.source[self.cur_pos..].len() <= self.cur_pos {
            return None;
        }

        let data_remain = self.source.len() - self.cur_pos;
        if data_remain < self.chunk_parms.min_chunk_size {
            let offset = self.cur_pos;
            let length = self.source[self.cur_pos..].len() - self.cur_pos;
            self.cur_pos = self.source.len();

            return Some(super::Chunk { offset, length });
        }

        let max_chunk_limit = std::cmp::min(
            self.chunk_parms.max_chunk_size,
            self.source[self.cur_pos..].len(),
        );
        let mut current_position = self.cur_pos;

        let mut window = [0u8; MAX_WIN_SIZE];
        let mut window_index = 0;
        let mut rolling_hash = 0u64;

        while current_position < max_chunk_limit {
            let byte = self.source[current_position];
            let out_byte = window[window_index] as usize;
            let out_value = self.out_map[out_byte];

            rolling_hash = ((rolling_hash * PRIME) & MASK)
                .wrapping_add(u64::from(byte))
                .wrapping_sub(out_value)
                & MASK;

            window[window_index] = byte;
            window_index = (window_index + 1) & self.win_mask;

            if current_position - self.cur_pos + 1 >= self.chunk_parms.min_chunk_size {
                let checksum = rolling_hash ^ self.ir[out_byte];
                if (checksum & self.cut_mask) == 0 {
                    let offset = self.cur_pos;
                    let length = current_position - self.cur_pos + 1;
                    self.cur_pos = current_position + 1;

                    return Some(super::Chunk { offset, length });
                }
            }
            current_position += 1;
        }

        let offset = self.cur_pos;
        let length = max_chunk_limit;
        self.cur_pos += length;

        Some(super::Chunk { offset, length })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generates test data of a specified length filled with a repeating pattern.
    fn generate_test_data(length: usize) -> Vec<u8> {
        (0..length).map(|i| (i % 1024) as u8).collect()
    }

    /// Tests RabinCDC with a basic input to ensure it creates any chunks.
    #[test]
    fn test_basic_chunk_creation() {
        let data = generate_test_data(3000);
        let chunker = RabinCDC::new(&data, 64, 64, 256, 1024);

        let chunks: Vec<_> = chunker.collect();
        assert!(!chunks.is_empty(), "Should create at least one chunk");
    }

    /// Tests RabinCDC to ensure chunks do not exceed the maximum chunk size.
    #[test]
    fn test_max_chunk_size() {
        let data = generate_test_data(5000);
        let chunker = RabinCDC::new(&data, 64, 64, 256, 1024);

        let chunks: Vec<_> = chunker.collect();
        assert!(
            chunks.iter().all(|chunk| chunk.length <= 1024),
            "All chunks must be <= 1024 bytes"
        );
    }

    /// Tests RabinCDC to ensure that chunk boundaries change with different polynomial primes.
    #[test]
    fn test_different_primes() {
        let data = generate_test_data(500);
        let chunker1 = RabinCDC::new(&data, 64, 64, 256, 1024);

        let chunker2 = RabinCDC::new(&data, 64, 64, 256, 1024);

        let chunks1: Vec<_> = chunker1.collect();
        let chunks2: Vec<_> = chunker2.collect();

        assert_eq!(
            chunks1, chunks2,
            "Chunks should be the same when using the same prime"
        );
    }

    /// Tests RabinCDC with very small data to check edge cases.
    #[test]
    fn test_small_data() {
        let data = generate_test_data(50);
        let chunker = RabinCDC::new(&data, 64, 64, 256, 1024);

        let chunks: Vec<_> = chunker.collect();
        assert_eq!(
            chunks.len(),
            1,
            "Should create exactly one chunk with small data"
        );
        assert_eq!(
            chunks[0].length, 50,
            "The single chunk should contain all data"
        );
    }

    /// Tests RabinCDC with an invalid window size (not a power of two).
    #[test]
    #[should_panic(expected = "Window size must be a power of two")]
    fn test_invalid_window_size() {
        let data = generate_test_data(1000);
        let _chunker = RabinCDC::new(&data, 50, 50, 100, 200);
    }

    /// Tests RabinCDC with a window size that is too large.
    #[test]
    #[should_panic(expected = "Window size out of valid range")]
    fn test_window_size_too_large() {
        let data = generate_test_data(1000);
        let _chunker = RabinCDC::new(&data, 2048, 50, 100, 200);
    }

    /// Tests RabinCDC initialization with zero window size.
    #[test]
    #[should_panic(expected = "Window size out of valid range")]
    fn test_zero_window_size() {
        let data = generate_test_data(1000);
        let _chunker = RabinCDC::new(&data, 0, 50, 100, 200);
    }
}
