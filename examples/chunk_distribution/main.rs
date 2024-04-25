use std::path::PathBuf;

use clap::Parser;

mod chunking;
mod dedup;
mod distribution;
mod opts;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = opts::Cli::parse();

    match &cli.command {
        // Handle distribution command
        opts::Command::Dist(cmd) => {
            // Check if the provided source path exists before proceeding
            if !cmd.source.exists() {
                return Err(Box::from("Provided source path doesn't exist"));
            }

            // Read data and split into chunks based on the algorithm specified
            let data_chunks = chunking::read_and_chunk_data(&cmd.source, &cmd.algo)?;

            // Determine the file name for the output plot
            let file_name = format!("{} distribution.png", cmd.algo);

            // Determine the output directory and construct the full file path
            let out_dir = cmd.out.as_ref().map_or_else(
                // Use the current directory if no output directory is provided
                || Ok(PathBuf::from(&file_name)),
                // Validate and prepare the output directory
                |out| {
                    if out.is_file() {
                        Err(Box::<dyn std::error::Error>::from(
                            "Provided path for saving plot is a file",
                        ))
                    } else {
                        // Ensure the directory exists
                        if !out.exists() {
                            std::fs::create_dir_all(out)?;
                        }
                        Ok(out.join(&file_name))
                    }
                },
            )?;

            // Generate the distribution plot and save it to the specified path
            distribution::build_distribution(&data_chunks, &cmd.algo, out_dir.as_path())
        }
        // Handle deduplication command
        opts::Command::Dedup(cmd) => {
            // Check if the provided paths exist and are not directories before proceeding
            if !cmd.original.exists() {
                return Err(Box::from("Provided path to original file doesn't exist"));
            }
            if !cmd.edited.exists() {
                return Err(Box::from("Provided path to edited file doesn't exist"));
            }
            if cmd.original.is_dir() {
                return Err(Box::from("Provided path to original file is a directory"));
            }
            if cmd.edited.is_dir() {
                return Err(Box::from("Provided path to edited file is a directory"));
            }

            // Read data and split into chunks based on the algorithm specified
            let hashes_original = chunking::read_and_chunk_data(&cmd.original, &cmd.algo)?
                .iter()
                .map(|chunk| chunk.hash)
                .collect::<Vec<_>>();

            // Read data and split into chunks based on the algorithm specified
            let hashes_edited = chunking::read_and_chunk_data(&cmd.edited, &cmd.algo)?
                .iter()
                .map(|chunk| chunk.hash)
                .collect::<Vec<_>>();

            let ratio = dedup::calculate_deduplication_ratio(&hashes_original, &hashes_edited);
            println!("Deduplication Ratio: X{:.2}", ratio);

            Ok(())
        }
    }
}
