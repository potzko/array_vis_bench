use std::time::Duration;
use crate::traits::log_traits::SortLog;

/// Comprehensive metrics collected from a sorting operation
#[derive(Debug, Clone, Default)]
pub struct BenchmarkMetrics {
    /// Total execution time
    pub duration: Duration,
    /// Number of comparison operations
    pub comparisons: usize,
    /// Number of swap operations
    pub swaps: usize,
    /// Total auxiliary memory allocated (in bytes)
    pub auxiliary_memory_bytes: usize,
    /// Number of auxiliary array allocations
    pub auxiliary_allocations: usize,
    /// Number of auxiliary array deallocations
    pub auxiliary_deallocations: usize,
    /// Peak memory usage (auxiliary memory at any point)
    pub peak_memory_bytes: usize,
}

impl BenchmarkMetrics {
    /// Create metrics from a list of sort logs
    pub fn from_logs<T: Copy + Ord>(logs: &[SortLog<T>], duration: Duration) -> Self {
        let mut metrics = BenchmarkMetrics {
            duration,
            ..Default::default()
        };

        let mut current_memory = 0;
        let mut peak_memory = 0;

        for log in logs {
            match log {
                // Count comparisons
                SortLog::CmpInArr { .. } |
                SortLog::CmpData { .. } |
                SortLog::CmpDataU { .. } |
                SortLog::CmpAcrossArrs { .. } => {
                    metrics.comparisons += 1;
                },
                
                // Count swaps
                SortLog::Swap { .. } => {
                    metrics.swaps += 1;
                },
                
                // Track auxiliary memory allocations
                SortLog::CreateAuxArr { length, .. } => {
                    let memory_used = length * std::mem::size_of::<usize>();
                    metrics.auxiliary_memory_bytes += memory_used;
                    metrics.auxiliary_allocations += 1;
                    current_memory += memory_used;
                    peak_memory = peak_memory.max(current_memory);
                },
                
                SortLog::CreateAuxArrT { length, .. } => {
                    // For generic types, we need to estimate the size
                    // This is a limitation - we can't know the exact size of T at runtime
                    // We'll use a conservative estimate based on usize size
                    let memory_used = length * std::mem::size_of::<usize>();
                    metrics.auxiliary_memory_bytes += memory_used;
                    metrics.auxiliary_allocations += 1;
                    current_memory += memory_used;
                    peak_memory = peak_memory.max(current_memory);
                },
                
                // Track memory deallocations
                SortLog::FreeAuxArr { .. } => {
                    metrics.auxiliary_deallocations += 1;
                    // Note: We can't easily track exact deallocation size from logs
                    // This is a limitation of the current logging system
                },
                
                // Other operations don't affect our metrics
                SortLog::Mark(_) |
                SortLog::WriteInArr { .. } |
                SortLog::WriteData { .. } |
                SortLog::WriteDataU { .. } => {}
            }
        }

        metrics.peak_memory_bytes = peak_memory;
        metrics
    }

    /// Calculate operations per second
    pub fn operations_per_second(&self) -> f64 {
        let total_ops = self.comparisons + self.swaps;
        if self.duration.as_secs_f64() > 0.0 {
            total_ops as f64 / self.duration.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Calculate memory efficiency (operations per byte of auxiliary memory)
    pub fn memory_efficiency(&self) -> f64 {
        if self.auxiliary_memory_bytes > 0 {
            (self.comparisons + self.swaps) as f64 / self.auxiliary_memory_bytes as f64
        } else {
            f64::INFINITY
        }
    }

    /// Get a summary string for display
    pub fn summary(&self) -> String {
        format!(
            "Duration: {:?}, Comparisons: {}, Swaps: {}, Aux Memory: {} bytes, Peak: {} bytes, Ops/sec: {:.2}",
            self.duration,
            self.comparisons,
            self.swaps,
            self.auxiliary_memory_bytes,
            self.peak_memory_bytes,
            self.operations_per_second()
        )
    }
}

/// Result of a single benchmark run
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// The sort algorithm name
    pub sort_name: String,
    /// Array size used
    pub array_size: usize,
    /// Whether the sort completed successfully
    pub success: bool,
    /// Performance metrics
    pub metrics: BenchmarkMetrics,
    /// Any error message if the sort failed
    pub error: Option<String>,
}

impl BenchmarkResult {
    /// Create a successful benchmark result
    pub fn success(sort_name: String, array_size: usize, metrics: BenchmarkMetrics) -> Self {
        Self {
            sort_name,
            array_size,
            success: true,
            metrics,
            error: None,
        }
    }

    /// Create a failed benchmark result
    pub fn failure(sort_name: String, array_size: usize, error: String) -> Self {
        Self {
            sort_name,
            array_size,
            success: false,
            metrics: BenchmarkMetrics::default(),
            error: Some(error),
        }
    }
}

/// Configuration for adaptive benchmarking
#[derive(Debug, Clone)]
pub struct AdaptiveBenchmarkConfig {
    /// Maximum time allowed for a single sort operation
    pub max_duration: Duration,
    /// Starting array size
    pub start_size: usize,
    /// Maximum array size to test
    pub max_size: usize,
    /// Growth factor for array size (e.g., 1.5 means 50% increase each time)
    pub growth_factor: f64,
    /// Number of warmup runs before timing
    pub warmup_runs: usize,
    /// Number of timing runs to average
    pub timing_runs: usize,
}

impl Default for AdaptiveBenchmarkConfig {
    fn default() -> Self {
        Self {
            max_duration: Duration::from_secs(5), // 5 seconds max
            start_size: 10,
            max_size: 10000,
            growth_factor: 1.5,
            warmup_runs: 2,
            timing_runs: 3,
        }
    }
}

/// Results from adaptive benchmarking
#[derive(Debug, Clone)]
pub struct AdaptiveBenchmarkResults {
    /// The sort algorithm name
    pub sort_name: String,
    /// Results for each array size tested
    pub results: Vec<BenchmarkResult>,
    /// The maximum size that completed within time limit
    pub max_successful_size: Option<usize>,
    /// The size that exceeded the time limit
    pub first_failure_size: Option<usize>,
}

impl AdaptiveBenchmarkResults {
    /// Get the best performance result (lowest time per element)
    pub fn best_performance(&self) -> Option<&BenchmarkResult> {
        self.results
            .iter()
            .filter(|r| r.success)
            .min_by(|a, b| {
                let a_efficiency = a.metrics.duration.as_secs_f64() / a.array_size as f64;
                let b_efficiency = b.metrics.duration.as_secs_f64() / b.array_size as f64;
                a_efficiency.partial_cmp(&b_efficiency).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Get summary statistics
    pub fn summary(&self) -> String {
        let successful_results: Vec<_> = self.results.iter().filter(|r| r.success).collect();
        let max_size = self.max_successful_size.unwrap_or(0);
        let failure_size = self.first_failure_size.unwrap_or(0);
        
        format!(
            "Sort: {}, Max Size: {}, First Failure: {}, Successful Runs: {}/{}",
            self.sort_name,
            max_size,
            failure_size,
            successful_results.len(),
            self.results.len()
        )
    }
}








