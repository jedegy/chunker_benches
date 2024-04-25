use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::opts::ChunkingAlgo;

/// Constant representing a kilobyte in bytes
const KB: usize = 1024;
/// Constant representing a megabyte in bytes
const MB: usize = 1024 * KB;
/// Size of the data block for reading from file in bytes and chunking
const SEGMENT_SIZE: usize = 20 * MB;

/// Represents a chunk of data
pub struct DataChunk {
    /// The data chunk
    pub data_chunk: Vec<u8>,
    /// The hash of the data chunk
    pub hash: blake3::Hash,
}

impl DataChunk {
    /// Create a new data chunk from the provided data.
    ///
    /// # Arguments
    ///
    /// * `data_chunk` - The data chunk.
    ///
    /// # Returns
    ///
    /// A new data chunk.
    fn new(data_chunk: Vec<u8>) -> Self {
        let hash = blake3::hash(&data_chunk);

        Self { data_chunk, hash }
    }
}

/// Read a block of data from the provided reader into the buffer.
/// This function will continue reading until the buffer is full or the reader returns EOF.
///
/// # Arguments
///
/// * `f` - The reader to read data from.
/// * `buf` - The buffer to read data into.
///
/// # Returns
///
/// The number of bytes read into the buffer.
pub fn read_block(
    f: &mut impl Read,
    mut buf: &mut [u8],
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut total = 0usize;
    while !buf.is_empty() {
        match f.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                total += n;
                buf = &mut buf[n..];
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        }
    }

    Ok(total)
}

/// Read the data from the provided file and chunk it using the provided algorithm.
///
/// # Arguments
///
/// * `path` - The path to the file to read.
/// * `algo` - The chunking algorithm to use.
///
/// # Returns
///
/// A vector of data chunks.
pub fn read_and_chunk_data(
    path: &Path,
    algo: &ChunkingAlgo,
) -> Result<Vec<DataChunk>, Box<dyn std::error::Error>> {
    // Open the file
    let file = File::open(path)?;
    // Create a buffered reader to read the file
    let mut f = std::io::BufReader::new(file);

    let mut total = 0;
    let mut segment = Vec::with_capacity(SEGMENT_SIZE);
    let mut chunks_data = Vec::new();
    let mut aligning = Vec::new();

    loop {
        let len = read_block(&mut f, &mut segment[aligning.len()..])?;
        if len == 0 {
            break;
        }
        total += len;

        // Chunk the reading data + aligning data from the previous iteration
        let mut chunks = chunk_data(algo, &segment[..len + aligning.len()]);

        // Put the last chunk in aligning
        if let Some(chunk) = chunks.pop() {
            let start_offset = chunk.offset;
            let chunk_slice = &segment[start_offset..start_offset + chunk.length];
            aligning = chunk_slice.to_vec();
        }

        // Form data chunks from the chunks
        chunks.into_iter().for_each(|chunk| {
            let start_offset = chunk.offset;
            let data_chunk = segment[start_offset..start_offset + chunk.length].to_vec();

            chunks_data.push(DataChunk::new(data_chunk));
        });

        // Copy the aligning data to the beginning of the segment
        if !aligning.is_empty() {
            segment[..aligning.len()].copy_from_slice(&aligning);
        }
    }

    // Check if last aligning left
    if !aligning.is_empty() {
        chunks_data.push(DataChunk::new(aligning));
    }

    println!("Total read {} bytes", total);
    Ok(chunks_data)
}

/// Chunk the provided source data using the provided algorithm.
///
/// # Arguments
///
/// * `algo` - The chunking algorithm to use.
/// * `source` - The source data to chunk.
///
/// # Returns
///
/// A vector of chunks.
fn chunk_data(algo: &ChunkingAlgo, source: &[u8]) -> Vec<chunker_benches::Chunk> {
    match algo {
        ChunkingAlgo::FixedSize(args) => {
            chunker_benches::FixedSizeChunking::new(source, args.chunk_size.get()).collect()
        }
        ChunkingAlgo::GearCdc(args) => fastcdc::ronomon::FastCDC::new(
            source,
            args.min_size.get(),
            args.avg_size.get(),
            args.max_size.get(),
        )
        .map(|chunk| chunker_benches::Chunk {
            offset: chunk.offset,
            length: chunk.length,
        })
        .collect(),
        ChunkingAlgo::FastCdc(args) => fastcdc::v2020::FastCDC::new(
            source,
            args.min_size.get() as u32,
            args.avg_size.get() as u32,
            args.max_size.get() as u32,
        )
        .map(|chunk| chunker_benches::Chunk {
            offset: chunk.offset,
            length: chunk.length,
        })
        .collect(),
    }
}
