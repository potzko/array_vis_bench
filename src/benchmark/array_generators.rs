use rand::Rng;

/// Extended array generation functions for comprehensive benchmarking
pub mod array_generators {
    use super::*;

    /// Generate a seesaw pattern array: [0, n-1, 1, n-2, 2, n-3, ...]
    pub fn get_seesaw_arr(length: usize) -> Vec<usize> {
        let mut arr = Vec::with_capacity(length);
        let mut left = 0;
        let mut right = length - 1;
        let mut use_left = true;
        
        for _ in 0..length {
            if use_left {
                arr.push(left);
                left += 1;
            } else {
                arr.push(right);
                right -= 1;
            }
            use_left = !use_left;
        }
        arr
    }

    /// Generate a partially sorted array (sorted_ratio between 0.0 and 1.0)
    pub fn get_partially_sorted_arr(length: usize, sorted_ratio: f64) -> Vec<usize> {
        let mut arr = crate::utils::array_gen::get_arr(length);
        let shuffle_count = ((1.0 - sorted_ratio) * length as f64) as usize;
        
        let mut rng = rand::thread_rng();
        for _ in 0..shuffle_count {
            let i = rng.gen_range(0..length);
            let j = rng.gen_range(0..length);
            arr.swap(i, j);
        }
        arr
    }

    /// Generate an array with many duplicates
    pub fn get_duplicate_heavy_arr(length: usize, unique_values: usize) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        let mut arr = Vec::with_capacity(length);
        
        for _ in 0..length {
            arr.push(rng.gen_range(0..unique_values));
        }
        arr
    }

    /// Generate a nearly sorted array with small perturbations
    pub fn get_nearly_sorted_arr(length: usize, perturbation_ratio: f64) -> Vec<usize> {
        let mut arr = crate::utils::array_gen::get_arr(length);
        let perturbation_count = (perturbation_ratio * length as f64) as usize;
        
        let mut rng = rand::thread_rng();
        for _ in 0..perturbation_count {
            let i = rng.gen_range(0..length);
            let j = rng.gen_range(0..length);
            arr.swap(i, j);
        }
        arr
    }

    /// Generate an array with specific distribution pattern
    pub fn get_distribution_arr(length: usize, pattern: DistributionPattern) -> Vec<usize> {
        match pattern {
            DistributionPattern::Random => crate::utils::array_gen::get_rand_arr(length),
            DistributionPattern::Ascending => crate::utils::array_gen::get_arr(length),
            DistributionPattern::Descending => crate::utils::array_gen::get_reversed_arr(length),
            DistributionPattern::Seesaw => get_seesaw_arr(length),
            DistributionPattern::PartiallySorted(ratio) => get_partially_sorted_arr(length, ratio as f64 / 100.0),
            DistributionPattern::DuplicateHeavy(unique) => get_duplicate_heavy_arr(length, unique),
            DistributionPattern::NearlySorted(ratio) => get_nearly_sorted_arr(length, ratio as f64 / 100.0),
        }
    }
}

/// Different array distribution patterns for benchmarking
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DistributionPattern {
    /// Completely random array
    Random,
    /// Already sorted (ascending)
    Ascending,
    /// Reverse sorted (descending)
    Descending,
    /// Seesaw pattern: [0, n-1, 1, n-2, ...]
    Seesaw,
    /// Partially sorted with given ratio (0.0 = random, 1.0 = sorted)
    PartiallySorted(u8), // Use u8 to represent ratio * 100 (0-100)
    /// Many duplicates with given number of unique values
    DuplicateHeavy(usize),
    /// Nearly sorted with given perturbation ratio
    NearlySorted(u8), // Use u8 to represent ratio * 100 (0-100)
}

impl DistributionPattern {
    /// Get a human-readable name for the pattern
    pub fn name(&self) -> &'static str {
        match self {
            DistributionPattern::Random => "Random",
            DistributionPattern::Ascending => "Ascending",
            DistributionPattern::Descending => "Descending",
            DistributionPattern::Seesaw => "Seesaw",
            DistributionPattern::PartiallySorted(_) => "Partially Sorted",
            DistributionPattern::DuplicateHeavy(_) => "Duplicate Heavy",
            DistributionPattern::NearlySorted(_) => "Nearly Sorted",
        }
    }

    /// Get all standard distribution patterns
    pub fn standard_patterns() -> Vec<DistributionPattern> {
        vec![
            DistributionPattern::Random,
            DistributionPattern::Ascending,
            DistributionPattern::Descending,
            DistributionPattern::Seesaw,
            DistributionPattern::PartiallySorted(80),
            DistributionPattern::DuplicateHeavy(10),
            DistributionPattern::NearlySorted(10),
        ]
    }
}
