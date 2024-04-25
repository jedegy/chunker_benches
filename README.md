# Overview

This project aims to provide a benchmarking tool for evaluating the throughput of various chunking algorithms.
Chunking algorithms are integral in data deduplication, content-defined chunking (CDC), and similar data processing
tasks.

# Introduction

Data chunking involves breaking down data into smaller, more manageable parts called chunks. These chunks play a crucial
role in tasks such as data deduplication, backup systems, and distributed file systems. Various algorithms exist for
chunking data, each with its own trade-offs in terms of computational complexity, memory usage, and effectiveness.

This project provides some tools for different chunking algorithms:

- **Fixed Size Chunking:** Splits data into fixed-size chunks.
- **Gear CDC:** A content-defined chunking algorithm that uses Gear rolling hash.
- **Fast CDC:** Optimized Gear content-defined chunking algorithm for speed.
- **Rabin Karp CDC:** A content-defined chunking algorithm that uses Rabin-Karp rolling hash. **Internal implementation
  doesn't work well.**

# Benchmarking

To run the chunking benchmarks, use the following command:

```shell
cargo bench --bench chunker_benches
```

This will execute the benchmarking process and display the throughput of each algorithm.

# Examples

Additionally, an example application demonstrates the usage of chunking algorithms by providing options to visualize
chunk distribution and calculate deduplication ratios.

## Usage

The example program supports two main commands:

1. **Dist**: Visualize chunk distribution.
2. **Dedup**: Show deduplication ratio.

### Dist Command

Visualize chunk distribution of a source dataset.

```shell
cargo run --example chunk_distribution dist [OPTIONS] --source <SOURCE>
```

**Options**

- `-s, --source <SOURCE>` - Path to the source dataset to be chunked.
- `-o, --out <OUT>` - Output directory for plots.
- `--algo <ALGO>` - Chunking algorithm to use. Available options are:

    - `fixed-size <CHUNK_SIZE>` - Fixed size chunks.

    - `gear-cdc <MIN_SIZE> <AVG_SIZE> <MAX_SIZE>` - Gear-based Content-Defined Chunking.

    - `fast-cdc <MIN_SIZE> <AVG_SIZE> <MAX_SIZE>` - Fast Content-Defined Chunking.

### Dedup Command

Show a deduplication ratio between original and modified datasets.

```shell
cargo run --example chunk_distribution dedup [OPTIONS] --original <ORIGINAL> --edited <EDITED>
```

**Options**

- `-o, --original <ORIGINAL>` - Path to the original dataset.
- `-e, --edited <EDITED>` - Path to the modified dataset.
- `--algo <ALGO>` - Chunking algorithm to use. Available options are the same as in the `Dist` command.