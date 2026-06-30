//! Standalone speed benchmark across ALL first-class kinds.
//!
//! For each category, times every registered algorithm's `run_with_input`
//! (median of `runs`) and prints a fastest-first table. Same timed-call style
//! as the original sort-only benchmark, generalised: a rotation / merge /
//! partition / small-sort / quick-select entry is just as much a single timed
//! `run_with_input` call as a sort is. Args: `[N] [runs]`.
use std::time::Instant;

use array_vis_bench_core::bench_registry::{
    primary_input, AlgorithmEntry, Category, RunConfig, ALGORITHMS,
};
use sort_logger::{NoOpLogger, SortLogger};

// ponytail: the old `buf.shuffle` was dead — `run_with_input` regenerates its
// input from the category's registry each call, so the external buffer + `rand`
// were never timed. Dropped.

const CATEGORIES: [Category; 6] = [
    Category::Sort,
    Category::QuickSelect,
    Category::Merge,
    Category::Partition,
    Category::SmallSort,
    Category::Rotation,
];

fn median_ns(times: &mut [u64]) -> u64 {
    times.sort_unstable();
    times[times.len() / 2]
}

fn fmt_ns(ns: u64) -> (f64, &'static str) {
    if ns < 1_000 {
        (ns as f64, "ns")
    } else if ns < 1_000_000 {
        (ns as f64 / 1_000.0, "µs")
    } else if ns < 1_000_000_000 {
        (ns as f64 / 1_000_000.0, "ms")
    } else {
        (ns as f64 / 1_000_000_000.0, "s ")
    }
}

fn bench_category(cat: Category, n: usize, runs: usize) {
    let entries: Vec<&'static AlgorithmEntry> =
        ALGORITHMS.iter().filter(|e| e.category == cat).collect();
    if entries.is_empty() {
        return;
    }
    let input = primary_input(cat);

    let mut results: Vec<(&'static str, u64)> = Vec::with_capacity(entries.len());
    for e in entries {
        // Clamp to the entry's contract cap — small sorts only run on inputs
        // up to their threshold (their input generator clamps too, this just
        // keeps the reported size honest).
        let size = e.max_input_size.map_or(n, |c| n.min(c));
        let cfg = RunConfig { size, seed: 0 };
        let mut times = Vec::with_capacity(runs);
        for _ in 0..runs {
            let t = Instant::now();
            (e.run_with_input)(input, &cfg, &mut NoOpLogger as &mut dyn SortLogger<usize>);
            times.push(t.elapsed().as_nanos() as u64);
        }
        results.push((e.name, median_ns(&mut times)));
    }
    results.sort_by_key(|&(_, ns)| ns);

    let name_w = results.iter().map(|(n, _)| n.len()).max().unwrap_or(20).max(20);
    println!("\n=== {} ({} variants) ===", cat.as_str(), results.len());
    println!("{:<width$}  {:>12}", "algorithm", "median", width = name_w);
    println!("{:-<width$}  {:->12}", "", "", width = name_w);
    for (name, ns) in &results {
        let (val, unit) = fmt_ns(*ns);
        println!("{:<width$}  {:>9.2} {unit}", name, val, width = name_w);
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let n: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(100_000);
    let runs: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(7);

    println!("Speed test  N={n}  runs={runs}");
    for cat in CATEGORIES {
        bench_category(cat, n, runs);
    }
}
