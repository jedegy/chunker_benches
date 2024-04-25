/// Fixed size chunking for data segmentation.
pub struct FixedSizeChunking<'a> {
    /// Data buffer to chunk.
    source: &'a [u8],
    /// Size of each chunk.
    chunk_size: usize,
    /// Current index position in data.
    current_index: usize,
}

impl<'a> FixedSizeChunking<'a> {
    /// Constructs a new `FixedSizeChunking`.
    ///
    /// # Arguments
    /// * `source` - Data buffer to be chunked.
    /// * `chunk_size` - Fixed size of each chunk.
    ///
    /// # Panics
    ///
    /// Panic if `chunk_size` is zero, as this would not allow for any meaningful chunking.
    pub fn new(source: &'a [u8], chunk_size: usize) -> Self {
        assert!(chunk_size > 0, "Chunk size must be greater than zero");

        Self {
            source,
            chunk_size,
            current_index: 0,
        }
    }
}

impl<'a> Iterator for FixedSizeChunking<'a> {
    type Item = super::Chunk;

    /// Computes the next chunk of fixed size.
    ///
    /// # Returns
    ///
    /// Returns a slice representing the next chunk if available, otherwise `None` if an end of data is reached.
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.source.len() {
            None
        } else {
            let length = if self.source.len() - self.current_index < self.chunk_size {
                self.source.len() - self.current_index
            } else {
                self.chunk_size
            };
            let offset = self.current_index;
            self.current_index += self.chunk_size;

            Some(super::Chunk { offset, length })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunks_of_equal_size() {
        let data = generate_test_data(10);
        let chunker = FixedSizeChunking::new(&data, 3);
        let chunks: Vec<_> = chunker.collect();

        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0].offset, 0);
        assert_eq!(chunks[0].length, 3);

        assert_eq!(chunks[1].offset, 3);
        assert_eq!(chunks[1].length, 3);

        assert_eq!(chunks[2].offset, 6);
        assert_eq!(chunks[2].length, 3);

        assert_eq!(chunks[3].offset, 9);
        assert_eq!(chunks[3].length, 1);
    }

    #[test]
    #[should_panic(expected = "Chunk size must be greater than zero")]
    fn test_zero_chunk_size() {
        let data = generate_test_data(10);
        let _chunker = FixedSizeChunking::new(&data, 0);
    }

    /// Helper function to generate test data.
    fn generate_test_data(length: usize) -> Vec<u8> {
        (0..length).map(|i| i as u8).collect()
    }
}
