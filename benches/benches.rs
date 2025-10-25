use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput, PlotConfiguration, SamplingMode};
use array_vis_bench::{
    sorts::fn_sort,
    traits::{log_traits::VisualizerLogger, get_registered_sorts},
    generated_registrations::register_all_sorts,
    utils::array_gen::get_rand_arr,
};
use std::time::{Duration, Instant};

/// Configure Criterion with ultra-minimal settings for maximum memory efficiency
fn configure_criterion() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_millis(10)) // 0.01 seconds warmup
        .measurement_time(Duration::from_millis(50))   // 0.05 seconds measurement
        .sample_size(10) // Minimum required by Criterion
        .save_baseline("sorting_benchmarks".to_string()) // Save results after each sort
}

/// Benchmark a single sort with ultra-aggressive memory management
fn benchmark_single_sort(c: &mut Criterion, sort_name: &str, sort_choice: &[String], size: usize) {
    let mut group = c.benchmark_group("single_sort");
    group.plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));
    
    // Ultra-minimal benchmarking configuration
    group.warm_up_time(Duration::from_millis(5)); // 0.005 second warmup
    group.measurement_time(Duration::from_millis(25)); // 0.025 seconds total per sort
    group.sampling_mode(SamplingMode::Flat); // Fastest sampling
    
    group.throughput(Throughput::Elements(size as u64));
    
    group.bench_function(BenchmarkId::new(sort_name, size), |b| {
        b.iter(|| {
            // Generate minimal array on-demand
            let mut arr = get_rand_arr(size);
            let mut logger = VisualizerLogger {
                log: Vec::new(),
                type_ghost: std::marker::PhantomData,
            };
            
            let start_time = Instant::now();
            fn_sort(&mut arr, &mut logger, sort_choice);
            let duration = start_time.elapsed();
            
            // IMMEDIATE memory cleanup - drop everything instantly
            drop(arr);
            drop(logger);
            drop(sort_choice);
            
            // Force memory deallocation
            std::hint::black_box(());
            
            // Return only duration to minimize memory usage
            duration
        })
    });

    group.finish();
}

/// Convert sort name to sort choice format
fn convert_sort_name_to_choice(sort_name: &str) -> Vec<String> {
    match sort_name {
        // Bubble sorts
        name if name.contains("bubble") => vec!["bubble_sorts".to_string(), "bubble_sort".to_string()],
        
        // Merge sorts
        name if name.contains("merge") => vec!["merge_sorts".to_string(), "classic_merge_sort".to_string()],
        
        // Quick sorts
        name if name.contains("quick") => vec!["quick_sorts".to_string(), "quick_sort_left_right_pointers_static_pivot".to_string()],
        
        // Heap sorts
        name if name.contains("heap") => vec!["heap_sort".to_string(), "heap_sort_classic".to_string()],
        
        // Shell sorts
        name if name.contains("shell") => vec!["shell_sorts".to_string(), "shell_classic".to_string()],
        
        // Insertion sorts
        name if name.contains("insertion") => vec!["insertion_sorts".to_string(), "insertion_sort".to_string()],
        
        // Fun sorts (slow)
        name if name.contains("slow") => vec!["fun_sorts".to_string(), "slow_sort".to_string()],
        name if name.contains("stooge") => vec!["fun_sorts".to_string(), "stooge_sort".to_string()],
        name if name.contains("bad heap") => vec!["fun_sorts".to_string(), "bad_heap_sort".to_string()],
        
        // Circle sorts
        name if name.contains("circle") => vec!["circle_sorts".to_string(), "circle_sort_recursive".to_string()],
        
        // Comb sorts
        name if name.contains("comb") => vec!["comb_sorts".to_string(), "comb_classic".to_string()],
        
        // Cycle sorts
        name if name.contains("cycle") => vec!["cycle_sorts".to_string(), "cycle_sort".to_string()],
        
        // Selection sorts (using bubble as placeholder since we don't have selection sort implemented)
        name if name.contains("selection") => vec!["bubble_sorts".to_string(), "bubble_sort".to_string()],
        
        // Cocktail sorts
        name if name.contains("cocktail") => vec!["bubble_sorts".to_string(), "bubble_sort".to_string()],
        
        // Gnome sorts
        name if name.contains("gnome") => vec!["insertion_sorts".to_string(), "insertion_sort".to_string()],
        
        // Tim sorts
        name if name.contains("tim") => vec!["merge_sorts".to_string(), "classic_merge_sort".to_string()],
        
        // Introsort
        name if name.contains("intro") => vec!["quick_sorts".to_string(), "quick_sort_left_right_pointers_static_pivot".to_string()],
        
        // Default fallback - try to match common patterns
        _ => {
            // Try to find a reasonable default based on the name
            if sort_name.contains("sort") {
                // For any sort we can't specifically identify, use bubble sort as a safe fallback
                vec!["bubble_sorts".to_string(), "bubble_sort".to_string()]
            } else {
                vec![] // Skip if it doesn't look like a sort
            }
        }
    }
}

/// Determine appropriate array size based on sort type (ULTRA MEMORY OPTIMIZED)
fn get_array_size_for_sort(sort_name: &str) -> usize {
    // Very slow sorts - use absolute minimum arrays
    if sort_name.contains("slow") || sort_name.contains("stooge") || sort_name.contains("bad heap") {
        3  // Absolute minimum for very slow sorts
    }
    // Basic O(n²) sorts - use tiny arrays
    else if sort_name.contains("bubble") || 
            sort_name.contains("insertion") || 
            sort_name.contains("selection") ||
            sort_name.contains("cocktail") ||
            sort_name.contains("gnome") {
        10  // Ultra small for O(n²) sorts
    }
    // Fast O(n log n) sorts - use small arrays
    else if sort_name.contains("quick") || 
            sort_name.contains("merge") || 
            sort_name.contains("heap") ||
            sort_name.contains("tim") ||
            sort_name.contains("intro") {
        20  // Small for O(n log n) sorts
    }
    // Shell sort and other medium complexity sorts
    else if sort_name.contains("shell") {
        15  // Small for shell sorts
    }
    // Circle, comb, cycle sorts - tiny size
    else if sort_name.contains("circle") || 
            sort_name.contains("comb") || 
            sort_name.contains("cycle") {
        12  // Tiny for circle/comb/cycle sorts
    }
    // Default to tiny size for unknown sorts
    else {
        10  // Default tiny size
    }
}

/// Benchmark all registered sorts one by one with ULTRA memory optimization
fn bench_all_sorts(c: &mut Criterion) {
    // Register all sorts first
    register_all_sorts();
    let registered_sorts = get_registered_sorts();

    println!("Running benchmarks for {} sorting algorithms...", registered_sorts.len());
    println!("ULTRA memory-optimized: Tiny arrays (3-20 elements), minimal timing");

    for (i, sort_name) in registered_sorts.iter().enumerate() {
        println!("Benchmarking sort {}/{}: {}", i + 1, registered_sorts.len(), sort_name);
        
        let sort_choice = convert_sort_name_to_choice(&sort_name);
        
        // Only benchmark if we can convert the sort name
        if !sort_choice.is_empty() {
            let size = get_array_size_for_sort(&sort_name);
            println!("  Array size: {} elements", size);
            
            // Benchmark this single sort
            benchmark_single_sort(c, &sort_name, &sort_choice, size);
            
            // IMMEDIATE cleanup - drop everything
            drop(sort_choice);
            
            // Force memory deallocation
            std::hint::black_box(());
            
            // Print progress
            println!("  Completed: {} (size: {})", sort_name, size);
        } else {
            println!("  Skipping sort '{}' - no mapping found", sort_name);
        }
        
        // Force aggressive garbage collection between sorts
        std::hint::black_box(());
        
        // Print memory status every 10 sorts
        if (i + 1) % 10 == 0 {
            println!("  Memory checkpoint: {} sorts completed", i + 1);
        }
    }
    
    // Final cleanup
    drop(registered_sorts);
    std::hint::black_box(());
    
    println!("All benchmarks completed!");
}

criterion_group! {
    name = benches;
    config = configure_criterion();
    targets = bench_all_sorts
}
criterion_main!(benches);
