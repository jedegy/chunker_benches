use std::fmt::Display;
use std::time::Duration;

use criterion::{
    BenchmarkGroup, black_box, Criterion, criterion_group, criterion_main, Throughput,
};
use criterion::measurement::WallTime;

/// Constant representing a kilobyte in bytes
const KB: usize = 1024;
/// Constant representing a megabyte in bytes
const MB: usize = 1024 * KB;

/// Minimum chunk size used in the benchmarks
const BENCH_MIN_CHUNK_SIZE: usize = 8 * KB;
/// Average chunk size used in the benchmarks
const BENCH_AVG_CHUNK_SIZE: usize = 10 * KB;
/// Maximum chunk size used in the benchmarks
const BENCH_MAX_CHUNK_SIZE: usize = 64 * KB;

/// Seed used for benchmark data generation
const SEED: u128 = 0xDEADBEEFCAFEF00DC0DEFACE99C0FFEEu128;
/// Size of the data block used in the benchmarks
const BENCH_DATA_SIZE: usize = 40 * MB;

/// Enum representing the various chunking algorithms.
enum Algorithm {
    Fixedsize,
    GearCDC,
    FastCDC,
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Algorithm::Fixedsize => "Fixed Size Chunking",
            Algorithm::GearCDC => "Gear Content Defined Chunking",
            Algorithm::FastCDC => "Fast Content Defined Chunking",
        };
        write!(f, "{}", str)
    }
}

/// Run the specified chunking algorithm on the provided data.
///
/// # Arguments
///
/// * `algo` - The chunking algorithm to use.
/// * `data` - The data to hash.
fn run_chunking_algorithm(group: &mut BenchmarkGroup<WallTime>, algo: &Algorithm, data: &[u8]) {
    match algo {
        Algorithm::Fixedsize => {
            run_fsc(group, &algo.to_string(), data);
        }
        Algorithm::GearCDC => {
            run_gearcdc(group, &algo.to_string(), data);
        }
        Algorithm::FastCDC => {
            run_fastcdc(group, &algo.to_string(), data);
        }
    }
}

/// Run the FixedSizeChunking algorithm on the provided data.
///
/// # Arguments
///
/// * `group` - The benchmark group to add the benchmark to.
/// * `name` - The name of the benchmark.
/// * `data` - The data to chunk.
fn run_fsc(group: &mut BenchmarkGroup<WallTime>, name: &str, data: &[u8]) {
    group.bench_function(name, |b| {
        b.iter(|| {
            let chunks: Vec<_> = chunker_benches::FixedSizeChunking::new(
                black_box(data),
                black_box(BENCH_AVG_CHUNK_SIZE),
            )
            .collect();
            black_box(chunks);
        })
    });
}

/// Run the GearCDC algorithm on the provided data.
///
/// # Arguments
///
/// * `group` - The benchmark group to add the benchmark to.
/// * `name` - The name of the benchmark.
/// * `data` - The data to chunk.
fn run_gearcdc(group: &mut BenchmarkGroup<WallTime>, name: &str, data: &[u8]) {
    group.bench_function(name, |b| {
        b.iter(|| {
            let chunks: Vec<_> = fastcdc::ronomon::FastCDC::new(
                black_box(data),
                black_box(BENCH_MIN_CHUNK_SIZE),
                black_box(BENCH_AVG_CHUNK_SIZE),
                black_box(BENCH_MAX_CHUNK_SIZE),
            )
            .collect();
            black_box(chunks);
        })
    });
}

/// Run the FastCDC algorithm on the provided data.
///
/// # Arguments
///
/// * `group` - The benchmark group to add the benchmark to.
/// * `name` - The name of the benchmark.
/// * `data` - The data to chunk.
fn run_fastcdc(group: &mut BenchmarkGroup<WallTime>, name: &str, data: &[u8]) {
    group.bench_function(name, |b| {
        b.iter(|| {
            let chunks: Vec<_> = fastcdc::v2020::FastCDC::new(
                black_box(data),
                black_box(BENCH_MIN_CHUNK_SIZE as u32),
                black_box(BENCH_AVG_CHUNK_SIZE as u32),
                black_box(BENCH_MAX_CHUNK_SIZE as u32),
            )
            .collect();
            black_box(chunks);
        })
    });
}

/// Main function for running the chunking algorithm benchmarks.
///
/// This function benchmarks the throughput of various chunking algorithms using the
/// generated data block.
///
/// # Arguments
///
/// * `c` - The criterion context used for benchmarking.
fn run_benchmark(c: &mut Criterion) {
    // Generate a data block for benchmarking
    let data_block = chunker_benches::generate_data_block(BENCH_DATA_SIZE, Some(SEED));

    // Benchmark the throughput of the various chunking algorithms
    let mut group = c.benchmark_group("chunkers-throughput");
    group.throughput(Throughput::Bytes(BENCH_DATA_SIZE as u64));
    group.measurement_time(Duration::from_secs(10));

    // Define the chunking algorithms to benchmark
    let algorithms = vec![Algorithm::Fixedsize, Algorithm::GearCDC, Algorithm::FastCDC];

    // Run the chunking algorithms
    algorithms.iter().for_each(|algo| {
        run_chunking_algorithm(&mut group, algo, &data_block);
    });
}

criterion_group!(benches, run_benchmark);
criterion_main!(benches);
