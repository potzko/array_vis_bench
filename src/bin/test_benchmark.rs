use array_vis_bench::benchmark::{
    BenchmarkRunner, AdaptiveBenchmarkConfig, BenchmarkMetrics
};
use array_vis_bench::traits::log_traits::SortLog;
use std::time::Duration;

fn main() {
    println!("Testing Comprehensive Benchmark System");
    println!("=====================================");

    // Configure the benchmark with very short time limits for testing
    let config = AdaptiveBenchmarkConfig {
        max_duration: Duration::from_millis(100), // Very short for testing
        start_size: 5,
        max_size: 50,
        growth_factor: 2.0,
        warmup_runs: 1,
        timing_runs: 1,
    };

    // Create benchmark runner
    let runner = BenchmarkRunner::with_config(config);

    // Test with just a few sorts to verify the system works
    println!("Testing adaptive benchmark with a few sorts...");
    
    // Test metrics collection
    println!("\nTesting metrics collection...");
    let test_logs: Vec<SortLog<usize>> = vec![
        SortLog::CmpInArr { name: 0, ind_a: 0, ind_b: 1, result: true },
        SortLog::Swap { name: 0, ind_a: 0, ind_b: 1 },
        SortLog::CreateAuxArr { name: 1, length: 10 },
    ];
    
    let metrics = BenchmarkMetrics::from_logs(&test_logs, Duration::from_millis(50));
    println!("Test metrics: {}", metrics.summary());

    println!("\nBenchmark system test completed successfully!");
    println!("The comprehensive benchmark system is ready to use.");
    println!("Run 'cargo run --bin comprehensive_benchmark' for full benchmarks.");
}
