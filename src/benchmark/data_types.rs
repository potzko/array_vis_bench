use std::cmp::Ordering;
use rand::Rng;

/// Different data types for benchmarking comparison performance
pub mod data_types {
    use super::*;

    /// Fast comparison type - small integer
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FastInt(pub u8);

    impl PartialOrd for FastInt {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for FastInt {
        fn cmp(&self, other: &Self) -> Ordering {
            self.0.cmp(&other.0)
        }
    }

    /// Slow comparison type - large struct with expensive comparison
    #[derive(Debug, Clone, Copy)]
    pub struct SlowCompare {
        pub data: [u8; 64], // Large struct
        pub checksum: u64,   // Expensive computation
    }

    impl PartialEq for SlowCompare {
        fn eq(&self, other: &Self) -> bool {
            // Simulate expensive comparison
            self.checksum == other.checksum && self.data == other.data
        }
    }

    impl Eq for SlowCompare {}

    impl PartialOrd for SlowCompare {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for SlowCompare {
        fn cmp(&self, other: &Self) -> Ordering {
            // Simulate expensive comparison by doing some computation
            let self_sum: u64 = self.data.iter().map(|&x| x as u64).sum();
            let other_sum: u64 = other.data.iter().map(|&x| x as u64).sum();
            
            // Add some artificial delay
            std::hint::black_box(self_sum);
            std::hint::black_box(other_sum);
            
            self.checksum.cmp(&other.checksum)
        }
    }

    /// Variable cost comparison type - string with variable length
    #[derive(Debug, Clone)]
    pub struct VariableCost(pub String);

    impl PartialEq for VariableCost {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    impl Eq for VariableCost {}

    impl PartialOrd for VariableCost {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for VariableCost {
        fn cmp(&self, other: &Self) -> Ordering {
            // String comparison cost varies with length
            self.0.cmp(&other.0)
        }
    }

    /// Generate arrays of different data types
    pub fn generate_fast_int_array(length: usize) -> Vec<FastInt> {
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| FastInt(rng.gen()))
            .collect()
    }

    pub fn generate_slow_compare_array(length: usize) -> Vec<SlowCompare> {
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|i| {
                let mut data = [0u8; 64];
                for j in 0..64 {
                    data[j] = ((i + j) % 256) as u8;
                }
                let checksum = data.iter().map(|&x| x as u64).sum();
                SlowCompare { data, checksum }
            })
            .collect()
    }

    pub fn generate_variable_cost_array(length: usize) -> Vec<VariableCost> {
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|i| {
                let string_length = rng.gen_range(1..=20);
                let value = format!("{:0width$}", i, width = string_length);
                VariableCost(value)
            })
            .collect()
    }
}

/// Different data type categories for benchmarking
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataTypeCategory {
    /// Fast comparison, small data
    FastSmall,
    /// Fast comparison, large data  
    FastLarge,
    /// Slow comparison, small data
    SlowSmall,
    /// Slow comparison, large data
    SlowLarge,
    /// Variable cost comparison
    VariableCost,
}

impl DataTypeCategory {
    /// Get a human-readable name for the category
    pub fn name(&self) -> &'static str {
        match self {
            DataTypeCategory::FastSmall => "Fast Small (u8)",
            DataTypeCategory::FastLarge => "Fast Large (u64)",
            DataTypeCategory::SlowSmall => "Slow Small (Complex u8)",
            DataTypeCategory::SlowLarge => "Slow Large (Complex u64)",
            DataTypeCategory::VariableCost => "Variable Cost (String)",
        }
    }

    /// Get all standard data type categories
    pub fn standard_categories() -> Vec<DataTypeCategory> {
        vec![
            DataTypeCategory::FastSmall,
            DataTypeCategory::FastLarge,
            DataTypeCategory::SlowSmall,
            DataTypeCategory::SlowLarge,
            DataTypeCategory::VariableCost,
        ]
    }
}
