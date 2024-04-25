use std::collections::HashSet;

/// Calculate a deduplication ratio between two vectors of hashes
///
/// # Arguments
///
/// * `vec1` - First vector of hashes
/// * `vec2` - Second vector of hashes
///
/// # Returns
///
/// * Deduplication ratio as a `f64`
pub fn calculate_deduplication_ratio(vec1: &[blake3::Hash], vec2: &[blake3::Hash]) -> f64 {
    let total_count = vec1.len() + vec2.len();
    let mut unique_hashes = HashSet::<blake3::Hash>::new();

    // Add all hashes from both vectors to the HashSet to find all unique hashes
    unique_hashes.extend(vec1.iter());
    unique_hashes.extend(vec2.iter());

    // Calculate deduplication ratio
    let unique_count = unique_hashes.len();
    total_count as f64 / unique_count as f64
}
