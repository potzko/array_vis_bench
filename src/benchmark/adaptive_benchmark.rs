use std::time::{Duration, Instant};
use crate::traits::log_traits::{SortLog, VisualizerLogger};
use crate::sorts::fn_sort;
use crate::utils::array_gen::{get_rand_arr, get_arr, get_reversed_arr, get_rand_arr_in_range};
use super::metrics::{BenchmarkMetrics, BenchmarkResult, AdaptiveBenchmarkConfig, AdaptiveBenchmarkResults};

/// Adaptive benchmarking system that increases array size until time limit is exceeded
pub struct AdaptiveBenchmark {
    config: AdaptiveBenchmarkConfig,
}

impl AdaptiveBenchmark {
    /// Create a new adaptive benchmark with default configuration
    pub fn new() -> Self {
        Self {
            config: AdaptiveBenchmarkConfig::default(),
        }
    }

    /// Create a new adaptive benchmark with custom configuration
    pub fn with_config(config: AdaptiveBenchmarkConfig) -> Self {
        Self { config }
    }

    /// Run adaptive benchmark for a specific sort algorithm
    pub fn benchmark_sort(&self, sort_name: &str, sort_choice: &[String]) -> AdaptiveBenchmarkResults {
        let mut results = Vec::new();
        let mut current_size = self.config.start_size;
        let mut max_successful_size = None;
        let mut first_failure_size = None;

        while current_size <= self.config.max_size {
            match self.benchmark_at_size(sort_name, sort_choice, current_size) {
                Ok(result) => {
                    if result.success {
                        max_successful_size = Some(current_size);
                        results.push(result);
                        
                        // Check if we should continue to larger sizes
                        if current_size >= self.config.max_size {
                            break;
                        }
                        
                        // Increase size for next iteration
                        current_size = ((current_size as f64) * self.config.growth_factor) as usize;
                        current_size = current_size.min(self.config.max_size);
                    } else {
                        first_failure_size = Some(current_size);
                        results.push(result);
                        break;
                    }
                }
                Err(error) => {
                    let failure_result = BenchmarkResult::failure(
                        sort_name.to_string(),
                        current_size,
                        error,
                    );
                    first_failure_size = Some(current_size);
                    results.push(failure_result);
                    break;
                }
            }
        }

        AdaptiveBenchmarkResults {
            sort_name: sort_name.to_string(),
            results,
            max_successful_size,
            first_failure_size,
        }
    }

    /// Benchmark a sort at a specific array size
    fn benchmark_at_size(
        &self,
        sort_name: &str,
        sort_choice: &[String],
        array_size: usize,
    ) -> Result<BenchmarkResult, String> {
        // Generate test array
        let mut arr = get_rand_arr(array_size);
        
        // Warmup runs
        for _ in 0..self.config.warmup_runs {
            let mut warmup_arr = arr.clone();
            let mut warmup_logger = VisualizerLogger {
                log: Vec::new(),
                type_ghost: std::marker::PhantomData,
            };
            
            // Run the sort without timing
            fn_sort(&mut warmup_arr, &mut warmup_logger, sort_choice);
        }

        // Timing runs
        let mut total_duration = Duration::new(0, 0);
        let mut all_logs = Vec::new();

        for _ in 0..self.config.timing_runs {
            let mut test_arr = arr.clone();
            let mut logger = VisualizerLogger {
                log: Vec::new(),
                type_ghost: std::marker::PhantomData,
            };

            let start_time = Instant::now();
            
            // Run the sort
            fn_sort(&mut test_arr, &mut logger, sort_choice);

            let duration = start_time.elapsed();
            total_duration += duration;
            all_logs.extend(logger.log);
        }

        // Calculate average duration
        let avg_duration = total_duration / self.config.timing_runs as u32;

        // Check if we exceeded the time limit
        if avg_duration > self.config.max_duration {
            return Ok(BenchmarkResult::failure(
                sort_name.to_string(),
                array_size,
                format!("Exceeded time limit: {:?} > {:?}", avg_duration, self.config.max_duration),
            ));
        }

        // Create metrics from the logs
        let metrics = BenchmarkMetrics::from_logs(&all_logs, avg_duration);

        Ok(BenchmarkResult::success(
            sort_name.to_string(),
            array_size,
            metrics,
        ))
    }
}

impl Default for AdaptiveBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

/// Run adaptive benchmarks for multiple sorts
pub fn run_adaptive_benchmarks(
    sort_configs: Vec<(String, Vec<String>)>,
    config: Option<AdaptiveBenchmarkConfig>,
) -> Vec<AdaptiveBenchmarkResults> {
    let benchmark = match config {
        Some(cfg) => AdaptiveBenchmark::with_config(cfg),
        None => AdaptiveBenchmark::new(),
    };

    let mut results = Vec::new();

    for (sort_name, sort_choice) in sort_configs {
        println!("Benchmarking {}...", sort_name);
        let result = benchmark.benchmark_sort(&sort_name, &sort_choice);
        println!("{}", result.summary());
        results.push(result);
    }

    results
}

/// Print a comparison table of all benchmark results
pub fn print_comparison_table(results: &[AdaptiveBenchmarkResults]) {
    println!("\n=== ADAPTIVE BENCHMARK RESULTS ===");
    println!("{:<25} {:<10} {:<10} {:<12} {:<12} {:<12}", 
             "Sort Algorithm", "Max Size", "Time Limit", "Comparisons", "Swaps", "Aux Memory");
    println!("{}", "-".repeat(100));

    for result in results {
        if let Some(best) = result.best_performance() {
            println!("{:<25} {:<10} {:<10.2} {:<12} {:<12} {:<12}",
                result.sort_name,
                result.max_successful_size.unwrap_or(0),
                best.metrics.duration.as_secs_f64(),
                best.metrics.comparisons,
                best.metrics.swaps,
                best.metrics.auxiliary_memory_bytes
            );
        } else {
            println!("{:<25} {:<10} {:<10} {:<12} {:<12} {:<12}",
                result.sort_name,
                "FAILED",
                "N/A",
                "N/A",
                "N/A",
                "N/A"
            );
        }
    }
}
