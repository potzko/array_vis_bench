/// Quick sort speed comparison — no criterion overhead.
///
/// Runs every registered sort a fixed number of times on random arrays,
/// reports median elapsed time, prints a table sorted fastest-first.
///
/// Usage:
///   cargo run --release --bin speed_test
///   cargo run --release --bin speed_test -- 500000
///   cargo run --release --bin speed_test -- 500000 20
///
/// Arguments (all optional, positional):
///   1. N         — array size         (default: 100_000)
///   2. runs      — timed runs / sort  (default: 7)

use std::time::Instant;

use array_vis_bench_full::bench_registry::{primary_input, ALGORITHMS, Category, RunConfig};
use array_vis_bench_full::traits::log_traits::NoOpLogger;
use array_vis_bench_full::utils::array_gen::get_rand_arr;
use rand::seq::SliceRandom;
use rand::thread_rng;
use sort_logger::SortLogger;

fn median_ns(times: &mut Vec<u64>) -> u64 {
    times.sort_unstable();
    times[times.len() / 2]
}

fn main() {
    let mut args = std::env::args().skip(1);
    let n: usize = args
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);
    let runs: usize = args
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(7);

    println!("Speed test  N={n}  runs_per_sort={runs}");
    println!();

    let mut rng = thread_rng();
    let mut buf = get_rand_arr(n);
    let mut results: Vec<(&'static str, u64)> = Vec::new();

    let cfg = RunConfig { size: n, seed: 0 };
    for entry in ALGORITHMS {
        // Only sorts have a "time how long to sort this array"
        // interpretation. Other categories run via `run_correctness`-style
        // harnesses that aren't a single timed call.
        if entry.category != Category::Sort {
            continue;
        }
        let mut times = Vec::with_capacity(runs);
        for _ in 0..runs {
            buf.shuffle(&mut rng);
            let t = Instant::now();
            let mut logger = NoOpLogger;
            (entry.run_with_input)(
                primary_input(Category::Sort),
                &cfg,
                &mut logger as &mut dyn SortLogger<usize>,
            );
            times.push(t.elapsed().as_nanos() as u64);
        }
        let med = median_ns(&mut times);
        results.push((entry.name, med));
    }

    // Sort fastest first
    results.sort_by_key(|&(_, ns)| ns);

    // Pretty-print table
    let name_w = results.iter().map(|(n, _)| n.len()).max().unwrap_or(20).max(20);
    println!("{:<width$}  {:>12}", "sort", "median", width = name_w);
    println!("{:-<width$}  {:->12}", "", "", width = name_w);

    for (name, ns) in &results {
        let (val, unit) = if *ns < 1_000 {
            (*ns as f64, "ns")
        } else if *ns < 1_000_000 {
            (*ns as f64 / 1_000.0, "µs")
        } else if *ns < 1_000_000_000 {
            (*ns as f64 / 1_000_000.0, "ms")
        } else {
            (*ns as f64 / 1_000_000_000.0, "s ")
        };
        println!("{:<width$}  {:>9.2} {unit}", name, val, width = name_w);
    }

    println!();
    println!("{} sorts timed", results.len());
}
