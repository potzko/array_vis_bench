use crate::traits::get_registered_sorts;
use crate::generated_registrations::register_all_sorts;
use crate::sorts::fn_sort;
use super::{
    AdaptiveBenchmark, AdaptiveBenchmarkConfig, AdaptiveBenchmarkResults,
    BenchmarkMetrics, BenchmarkResult, print_comparison_table,
    DistributionPattern, DataTypeCategory,
    array_generators::array_generators::get_distribution_arr,
    data_types::data_types::{
        generate_fast_int_array, generate_slow_compare_array, generate_variable_cost_array,
        FastInt, SlowCompare, VariableCost
    }
};

/// Comprehensive benchmark runner that handles all types of benchmarks
pub struct BenchmarkRunner {
    adaptive_config: AdaptiveBenchmarkConfig,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner with default configuration
    pub fn new() -> Self {
        Self {
            adaptive_config: AdaptiveBenchmarkConfig::default(),
        }
    }

    /// Create a new benchmark runner with custom configuration
    pub fn with_config(adaptive_config: AdaptiveBenchmarkConfig) -> Self {
        Self { adaptive_config }
    }

    /// Run comprehensive benchmarks for all registered sorts
    pub fn run_all_benchmarks(&self) -> BenchmarkSuiteResults {
        // Register all sorts first
        register_all_sorts();
        let registered_sorts = get_registered_sorts();

        println!("Running comprehensive benchmarks for {} sorting algorithms...", registered_sorts.len());

        let mut results = BenchmarkSuiteResults::new();

        // 1. Adaptive benchmarks (time-based scaling)
        println!("\n=== ADAPTIVE BENCHMARKS ===");
        let adaptive_results = self.run_adaptive_benchmarks(&registered_sorts);
        results.adaptive_results = adaptive_results;

        // 2. Distribution benchmarks
        println!("\n=== DISTRIBUTION BENCHMARKS ===");
        let distribution_results = self.run_distribution_benchmarks(&registered_sorts);
        results.distribution_results = distribution_results;

        // 3. Data type benchmarks
        println!("\n=== DATA TYPE BENCHMARKS ===");
        let datatype_results = self.run_datatype_benchmarks(&registered_sorts);
        results.datatype_results = datatype_results;

        results
    }

    /// Run adaptive benchmarks for all sorts
    fn run_adaptive_benchmarks(&self, sort_names: &[String]) -> Vec<AdaptiveBenchmarkResults> {
        let benchmark = AdaptiveBenchmark::with_config(self.adaptive_config.clone());
        let mut results = Vec::new();

        for sort_name in sort_names {
            println!("Running adaptive benchmark for: {}", sort_name);
            
            // Convert sort name to sort choice format
            let sort_choice = self.convert_sort_name_to_choice(sort_name);
            
            let result = benchmark.benchmark_sort(sort_name, &sort_choice);
            results.push(result);
        }

        print_comparison_table(&results);
        results
    }

    /// Run distribution benchmarks for all sorts
    fn run_distribution_benchmarks(&self, sort_names: &[String]) -> DistributionBenchmarkResults {
        let patterns = DistributionPattern::standard_patterns();
        let mut results = DistributionBenchmarkResults::new();

        for pattern in &patterns {
            println!("Testing distribution pattern: {}", pattern.name());
            let mut pattern_results = Vec::new();

            for sort_name in sort_names {
                let sort_choice = self.convert_sort_name_to_choice(sort_name);
                let result = self.benchmark_with_distribution(sort_name, &sort_choice, pattern, 1000);
                pattern_results.push(result);
            }

            results.add_pattern_results(pattern.clone(), pattern_results);
        }

        results
    }

    /// Run data type benchmarks for all sorts
    fn run_datatype_benchmarks(&self, sort_names: &[String]) -> DataTypeBenchmarkResults {
        let categories = DataTypeCategory::standard_categories();
        let mut results = DataTypeBenchmarkResults::new();

        for category in &categories {
            println!("Testing data type category: {}", category.name());
            let mut category_results = Vec::new();

            for sort_name in sort_names {
                let sort_choice = self.convert_sort_name_to_choice(sort_name);
                let result = self.benchmark_with_datatype(sort_name, &sort_choice, category, 1000);
                category_results.push(result);
            }

            results.add_category_results(category.clone(), category_results);
        }

        results
    }

    /// Benchmark a sort with a specific distribution pattern
    fn benchmark_with_distribution(
        &self,
        sort_name: &str,
        sort_choice: &[String],
        pattern: &DistributionPattern,
        array_size: usize,
    ) -> BenchmarkResult {
        let arr = get_distribution_arr(array_size, pattern.clone());
        self.run_single_benchmark(sort_name, sort_choice, arr)
    }

    /// Benchmark a sort with a specific data type
    fn benchmark_with_datatype(
        &self,
        sort_name: &str,
        sort_choice: &[String],
        category: &DataTypeCategory,
        array_size: usize,
    ) -> BenchmarkResult {
        match category {
            DataTypeCategory::FastSmall => {
                let arr = generate_fast_int_array(array_size);
                self.run_single_benchmark_generic(sort_name, sort_choice, arr)
            }
            DataTypeCategory::FastLarge => {
                let arr: Vec<u64> = (0..array_size).map(|i| i as u64).collect();
                self.run_single_benchmark_generic(sort_name, sort_choice, arr)
            }
            DataTypeCategory::SlowSmall => {
                let arr = generate_slow_compare_array(array_size);
                self.run_single_benchmark_generic(sort_name, sort_choice, arr)
            }
            DataTypeCategory::SlowLarge => {
                let arr = generate_slow_compare_array(array_size);
                self.run_single_benchmark_generic(sort_name, sort_choice, arr)
            }
            DataTypeCategory::VariableCost => {
                // For now, skip variable cost as it doesn't implement Copy
                // This would need a different approach for non-Copy types
                BenchmarkResult::failure(
                    sort_name.to_string(),
                    array_size,
                    "VariableCost not supported (doesn't implement Copy)".to_string(),
                )
            }
        }
    }

    /// Run a single benchmark with usize array
    fn run_single_benchmark(
        &self,
        sort_name: &str,
        sort_choice: &[String],
        mut arr: Vec<usize>,
    ) -> BenchmarkResult {
        use crate::traits::log_traits::VisualizerLogger;
        use std::time::Instant;

        let start_time = Instant::now();
        let mut logger = VisualizerLogger {
            log: Vec::new(),
            type_ghost: std::marker::PhantomData,
        };

        fn_sort(&mut arr, &mut logger, sort_choice);

        let duration = start_time.elapsed();
        let metrics = BenchmarkMetrics::from_logs(&logger.log, duration);
        BenchmarkResult::success(sort_name.to_string(), arr.len(), metrics)
    }

    /// Run a single benchmark with generic array
    fn run_single_benchmark_generic<T: Ord + Copy>(
        &self,
        sort_name: &str,
        sort_choice: &[String],
        mut arr: Vec<T>,
    ) -> BenchmarkResult {
        use crate::traits::log_traits::VisualizerLogger;
        use std::time::Instant;

        let start_time = Instant::now();
        let mut logger = VisualizerLogger {
            log: Vec::new(),
            type_ghost: std::marker::PhantomData,
        };

        fn_sort(&mut arr, &mut logger, sort_choice);

        let duration = start_time.elapsed();
        let metrics = BenchmarkMetrics::from_logs(&logger.log, duration);
        BenchmarkResult::success(sort_name.to_string(), arr.len(), metrics)
    }

    /// Convert sort name to sort choice format (simplified version)
    fn convert_sort_name_to_choice(&self, sort_name: &str) -> Vec<String> {
        // This is a simplified version - you might want to use your existing create_sort_choice function
        // For now, we'll use a basic mapping
        match sort_name {
            name if name.contains("bubble") => vec!["bubble_sorts".to_string(), "bubble_sort".to_string()],
            name if name.contains("merge") => vec!["merge_sorts".to_string(), "classic_merge_sort".to_string()],
            name if name.contains("quick") => vec!["quick_sorts".to_string(), "quick_sort_left_right_pointers_static_pivot".to_string()],
            name if name.contains("heap") => vec!["heap_sort".to_string(), "heap_sort_classic".to_string()],
            name if name.contains("shell") => vec!["shell_sorts".to_string(), "shell_classic".to_string()],
            name if name.contains("insertion") => vec!["insertion_sorts".to_string(), "insertion_sort".to_string()],
            name if name.contains("slow") => vec!["fun_sorts".to_string(), "slow_sort".to_string()],
            name if name.contains("stooge") => vec!["fun_sorts".to_string(), "stooge_sort".to_string()],
            _ => vec!["bubble_sorts".to_string(), "bubble_sort".to_string()], // Default fallback
        }
    }
}

/// Results from a comprehensive benchmark suite
#[derive(Debug, Clone)]
pub struct BenchmarkSuiteResults {
    pub adaptive_results: Vec<AdaptiveBenchmarkResults>,
    pub distribution_results: DistributionBenchmarkResults,
    pub datatype_results: DataTypeBenchmarkResults,
}

impl BenchmarkSuiteResults {
    fn new() -> Self {
        Self {
            adaptive_results: Vec::new(),
            distribution_results: DistributionBenchmarkResults::new(),
            datatype_results: DataTypeBenchmarkResults::new(),
        }
    }

    /// Print a comprehensive summary of all benchmark results
    pub fn print_summary(&self) {
        println!("\n=== COMPREHENSIVE BENCHMARK SUMMARY ===");
        
        println!("\nAdaptive Benchmarks:");
        print_comparison_table(&self.adaptive_results);
        
        println!("\nDistribution Benchmarks:");
        self.distribution_results.print_summary();
        
        println!("\nData Type Benchmarks:");
        self.datatype_results.print_summary();
    }
}

/// Results from distribution pattern benchmarks
#[derive(Debug, Clone)]
pub struct DistributionBenchmarkResults {
    pub pattern_results: std::collections::HashMap<DistributionPattern, Vec<BenchmarkResult>>,
}

impl DistributionBenchmarkResults {
    fn new() -> Self {
        Self {
            pattern_results: std::collections::HashMap::new(),
        }
    }

    fn add_pattern_results(&mut self, pattern: DistributionPattern, results: Vec<BenchmarkResult>) {
        self.pattern_results.insert(pattern, results);
    }

    fn print_summary(&self) {
        for (pattern, results) in &self.pattern_results {
            println!("Pattern: {}", pattern.name());
            // Print top 3 fastest sorts for this pattern
            let mut sorted_results = results.clone();
            sorted_results.sort_by(|a, b| a.metrics.duration.cmp(&b.metrics.duration));
            
            for (i, result) in sorted_results.iter().take(3).enumerate() {
                println!("  {}: {} - {:?}", i + 1, result.sort_name, result.metrics.duration);
            }
        }
    }
}

/// Results from data type benchmarks
#[derive(Debug, Clone)]
pub struct DataTypeBenchmarkResults {
    pub category_results: std::collections::HashMap<DataTypeCategory, Vec<BenchmarkResult>>,
}

impl DataTypeBenchmarkResults {
    fn new() -> Self {
        Self {
            category_results: std::collections::HashMap::new(),
        }
    }

    fn add_category_results(&mut self, category: DataTypeCategory, results: Vec<BenchmarkResult>) {
        self.category_results.insert(category, results);
    }

    fn print_summary(&self) {
        for (category, results) in &self.category_results {
            println!("Category: {}", category.name());
            // Print top 3 fastest sorts for this category
            let mut sorted_results = results.clone();
            sorted_results.sort_by(|a, b| a.metrics.duration.cmp(&b.metrics.duration));
            
            for (i, result) in sorted_results.iter().take(3).enumerate() {
                println!("  {}: {} - {:?}", i + 1, result.sort_name, result.metrics.duration);
            }
        }
    }
}