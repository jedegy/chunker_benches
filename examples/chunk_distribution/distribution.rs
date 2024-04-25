use std::collections::HashMap;
use std::path::Path;

use plotters::prelude::*;

use crate::chunking::DataChunk;
use crate::opts::ChunkingAlgo;

/// Build a distribution of chunk sizes and save it to a file using the provided
/// data chunks and chunking algorithm.
///
/// # Arguments
///
/// * `data_chunks` - A slice of `DataChunk` instances
/// * `algo` - The chunking algorithm used to generate the chunks
/// * `out_file` - The path to save the distribution plot
///
/// # Returns
///
/// A result indicating success or failure.
pub fn build_distribution(
    data_chunks: &[DataChunk],
    algo: &ChunkingAlgo,
    out_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build a distribution of chunk sizes
    let mut distribution: HashMap<u64, u64> = HashMap::default();
    data_chunks.iter().for_each(|chunk| {
        let size = chunk.data_chunk.len();
        let count = distribution.entry(size as u64).or_insert(0);
        *count += 1;
    });

    // Find the maximum and second maximum number of chunks
    let mut max_number_chunks = 0;
    let mut second_max_number_chunks = 0;
    for &count in distribution.values() {
        if count > max_number_chunks {
            second_max_number_chunks = max_number_chunks;
            max_number_chunks = count;
        } else if count > second_max_number_chunks {
            second_max_number_chunks = count;
        }
    }

    // Calculate the total number of chunks
    let total_number_chunks = distribution.values().sum::<u64>();

    // Calculate the percentage of the maximum chunk size
    let percentage_max_chunk = if total_number_chunks > 0 {
        (max_number_chunks as f64 / total_number_chunks as f64) * 100.0
    } else {
        0.0
    };

    // Determine the maximum chunk size based on the chunking algorithm
    let max_x = match algo {
        ChunkingAlgo::FixedSize(args) => args.chunk_size.get(),
        ChunkingAlgo::GearCdc(args) => args.max_size.get(),
        ChunkingAlgo::FastCdc(args) => args.max_size.get(),
    };

    let title = format!(
        "{} distribution - {:.2}% of maximum chunk size",
        algo, percentage_max_chunk
    );

    // Draw the distribution plot
    draw_distribution(
        out_file,
        &title,
        data_chunks,
        0,
        max_x,
        0,
        second_max_number_chunks as usize,
    )
}

/// Draw a distribution plot of chunk sizes and save it to a file.
///
/// # Arguments
///
/// * `out_file` - The path to save the distribution plot
/// * `title` - The title of the plot
/// * `data_chunks` - A slice of `DataChunk` instances
/// * `min_x` - The minimum x-axis value
/// * `max_x` - The maximum x-axis value
/// * `min_y` - The minimum y-axis value
/// * `max_y` - The maximum y-axis value
///
/// # Returns
///
/// A result indicating success or failure.
pub fn draw_distribution(
    out_file: &Path,
    title: &str,
    data_chunks: &[DataChunk],
    min_x: usize,
    max_x: usize,
    min_y: usize,
    max_y: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(out_file, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(45)
        .y_label_area_size(50)
        .margin(5)
        .caption(title, ("sans-serif", 20.0))
        .build_cartesian_2d((min_x..max_x).into_segmented(), min_y..max_y)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc("Count")
        .x_desc("Size")
        .axis_desc_style(("sans-serif", 16))
        .draw()?;

    // Build a vector of data for the histogram
    let data: Vec<usize> = data_chunks
        .iter()
        .filter_map(|chunk| {
            if chunk.data_chunk.len() != max_x {
                Some(chunk.data_chunk.len())
            } else {
                None
            }
        })
        .collect();

    chart.draw_series(
        Histogram::vertical(&chart)
            .style(BLUE.mix(0.5).filled())
            .data(data.iter().map(|x: &usize| (*x, 1))),
    )?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file!");
    println!("Result has been saved to {}", out_file.to_str().unwrap());

    Ok(())
}
