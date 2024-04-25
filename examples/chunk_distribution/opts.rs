use std::fmt::{Display, Formatter};
use std::num::NonZeroUsize;
use std::path::PathBuf;

/// Global program options
#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// A command to execute
    #[command(subcommand)]
    pub command: Command,
}

/// Main program commands
#[derive(clap::Subcommand)]
pub enum Command {
    /// Visualize chunk distribution
    Dist(DistCmd),
    /// Show a deduplication ratio
    Dedup(DedupCmd),
}

/// Arguments for the `Dist` command
#[derive(clap::Args)]
pub struct DistCmd {
    /// Source dataset to be chunked
    #[arg(short, long)]
    pub source: PathBuf,

    /// Output directory for plots
    #[arg(short, long)]
    pub out: Option<PathBuf>,

    /// Chunking algorithm to use
    #[command(subcommand)]
    pub algo: ChunkingAlgo,
}

/// Arguments for the `Dedup` command
#[derive(clap::Args)]
pub struct DedupCmd {
    /// Path to the original dataset
    #[arg(short, long)]
    pub original: PathBuf,

    /// Path to the modified dataset
    #[arg(short, long)]
    pub edited: PathBuf,

    /// Chunking algorithm to use
    #[command(subcommand)]
    pub algo: ChunkingAlgo,
}

/// Chunking algorithms available
#[derive(clap::Subcommand)]
pub enum ChunkingAlgo {
    /// Fixed size chunks
    FixedSize(FixedSizeArgs),
    /// Gear-based Content-Defined Chunking
    GearCdc(GearCdcArgs),
    /// Fast Content-Defined Chunking
    FastCdc(FastCdcArgs),
}

/// Parameters for fixed size chunking algorithm
#[derive(clap::Args)]
pub struct FixedSizeArgs {
    /// Size of each chunk
    #[arg(long, value_parser = parse_humansize_nonzero_large)]
    pub chunk_size: NonZeroUsize,
}

/// Parameters for Gear CDC
#[derive(clap::Args)]
pub struct GearCdcArgs {
    /// Minimum chunk size
    #[arg(long, value_parser = parse_humansize_nonzero_large)]
    pub min_size: NonZeroUsize,
    /// Average chunk size
    #[arg(long, value_parser = parse_humansize_nonzero_large)]
    pub avg_size: NonZeroUsize,
    /// Maximum chunk size
    #[arg(long, value_parser = parse_humansize_nonzero_large)]
    pub max_size: NonZeroUsize,
}

/// Parameters for Fast CDC
#[derive(clap::Args)]
pub struct FastCdcArgs {
    /// Minimum chunk size
    #[arg(long, value_parser = parse_humansize_nonzero_large)]
    pub min_size: NonZeroUsize,
    /// Average chunk size
    #[arg(long, value_parser = parse_humansize_nonzero_large)]
    pub avg_size: NonZeroUsize,
    /// Maximum chunk size
    #[arg(long, value_parser = parse_humansize_nonzero_large)]
    pub max_size: NonZeroUsize,
}

impl Display for ChunkingAlgo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ChunkingAlgo::FixedSize(_) => "Fixed Size Chunking",
            ChunkingAlgo::GearCdc(_) => "Gear CDC Chunking",
            ChunkingAlgo::FastCdc(_) => "Fast CDC Chunking",
        };
        write!(f, "{}", str)
    }
}

/// Parse a string containing large positive size value with optional unit prefix
///
/// Parse a string containing a size which value in bytes does not exceed [`u64::MAX`] into
/// an integer. The string must represent a positive decimal integer number with optional
/// one-letter unit suffix, which directly follows the digits (i.e., is not separated from them
/// by any other character, including whitespace).
///
/// If the number contains a suffix, it is interpreted as a size in corresponding units:
///
/// * 'k' or 'K' - KiB (=1024 bytes)
/// * 'm' or 'M' - MiB (=1024 KiB)
/// * 'g' or 'G' - GiB (=1024 MiB)
/// * 't' or 'T' - TiB (=1024 GiB)
///
/// Otherwise, it is interpreted as a size in bytes.
///
/// If the product of the number and the unit size in bytes exceeds [`u64::MAX`], the result is
/// capped to [`u64::MAX`].
///
/// On success, the function returns the parsing result as [`NonZeroUsize`].
/// On error, it returns an error message describing the parsing failure.
pub fn parse_humansize_nonzero_large(mut source: &str) -> Result<NonZeroUsize, String> {
    use std::str::FromStr as _;

    const KB: usize = 1024;
    const MB: usize = 1024 * KB;
    const GB: usize = 1024 * MB;
    const TB: usize = 1024 * GB;

    let factor = match source
        .bytes()
        .last()
        .map(|b| (b as char).to_ascii_lowercase())
    {
        Some('k') => Some(KB),
        Some('m') => Some(MB),
        Some('g') => Some(GB),
        Some('t') => Some(TB),
        _ => None,
    };

    let factor = match factor {
        Some(factor) => {
            source = &source[..source.len() - 1];
            factor
        }
        None => 1,
    };

    let factor = NonZeroUsize::try_from(factor).unwrap();

    let parsed = NonZeroUsize::from_str(source).map_err(|err| {
        format!(
            "{}, expected positive decimal number with optional unit suffix (k,K,m,M,g,G,t,T)",
            err
        )
    })?;

    Ok(parsed.saturating_mul(factor))
}
