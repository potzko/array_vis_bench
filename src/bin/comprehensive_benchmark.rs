use array_vis_bench::benchmark::{
    BenchmarkRunner, AdaptiveBenchmarkConfig, BenchmarkSuiteResults
};
use std::time::Duration;

fn main() {
    println!("Comprehensive Sorting Algorithm Benchmark Suite");
    println!("================================================");

    // Configure the benchmark
    let config = AdaptiveBenchmarkConfig {
        max_duration: Duration::from_secs(10), // 10 seconds max per sort
        start_size: 10,
        max_size: 5000,
        growth_factor: 1.5,
        warmup_runs: 2,
        timing_runs: 3,
    };

    // Create benchmark runner
    let runner = BenchmarkRunner::with_config(config);

    // Run comprehensive benchmarks
    let results = runner.run_all_benchmarks();

    // Print comprehensive summary
    results.print_summary();

    println!("\nBenchmark completed successfully!");
}








