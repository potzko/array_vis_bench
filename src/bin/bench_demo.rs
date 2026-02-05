use std::time::Instant;

use array_vis_bench::{
    bench_registry::{self, SortBenchEntry},
    utils::{array_gen::get_rand_arr, check_utils},
};

fn main() {
    // Simple demo benchmark over all registered sorts (usize + NoOpLogger)
    let size = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(5000);

    println!("Bench demo: {} elements\n", size);

    // Base array, cloned per sort to keep the same workload
    let base = get_rand_arr(size);

    // Iterate over compile-time registered entries (no trait objects)
    bench_registry::for_each(|entry: &'static SortBenchEntry| {
        // Clone base array for fairness
        let mut arr = base.clone();
        let start = Instant::now();
        (entry.run)(&mut arr);
        let elapsed = start.elapsed();

        let ok = check_utils::is_sorted(&arr);
        println!(
            "- {:>9} | {:<6} | {:>8.3} ms | sorted: {}",
            entry.big_o,
            if entry.stable { "stable" } else { "unstbl" },
            elapsed.as_secs_f64() * 1_000.0,
            ok
        );
        // Optional: print the name for clarity, comment out if noisy
        println!("  name: {}", entry.name);
    });
}
