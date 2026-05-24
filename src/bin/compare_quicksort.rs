//! Compare a hand-rolled u64 quicksort against the system's generated
//! equivalent.
//!
//! Both sorts use:
//!   - Lomuto partition (left-left two-pointer scan)
//!   - First-element pivot
//!   - Switch to linear insertion sort at len <= 32
//!
//! The system version goes through `SortLogger` (instantiated with
//! `NoOpLogger`), so this measures whether the trait/inlining tax is zero
//! in practice.
//!
//! Usage:
//!   cargo run --release --bin compare_quicksort
//!   cargo run --release --bin compare_quicksort -- 1_000_000 31
//!
//! Build with `target-cpu=native` for fair numbers — the project's
//! `.cargo/config.toml` sets this by default, but if you build with a
//! different toolchain or shell that ignores it:
//!   RUSTFLAGS="-C target-cpu=native" cargo run --release --bin compare_quicksort
//! Without it, baseline x86_64 code-gen leaves the system version ~5%
//! behindeven though the asm is otherwise equivalent.

use std::time::Instant;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use array_vis_bench::sorts::quick_sorts::partitions::Lomuto;
use array_vis_bench::sorts::quick_sorts::pivot_selectors::FirstElement;
use array_vis_bench::sorts::quick_sorts::quick_sort::QuickSort;
use array_vis_bench::traits::log_traits::NoOpLogger;
use array_vis_bench::utils::small_sort::{InsertionSmallSort, LinearInsertion};

const THRESHOLD: usize = 32;

/// Hand-rolled u64 quicksort matching the system combo exactly:
/// Lomuto + first-element pivot + insertion-sort under-32 threshold.
fn hand_quicksort(arr: &mut [u64]) {
    if arr.len() <= THRESHOLD {
        // Linear insertion sort
        for i in 1..arr.len() {
            let mut j = i;
            while j > 0 && arr[j] < arr[j - 1] {
                arr.swap(j, j - 1);
                j -= 1;
            }
        }
        return;
    }

    // Lomuto partition with first-element pivot.
    // Move pivot to the end, scan forward, swap small-or-equal elements to the front.
    let len = arr.len();
    arr.swap(0, len - 1);
    let pivot = arr[len - 1];

    let mut small = 0;
    for i in 0..len - 1 {
        if arr[i] <= pivot {
            arr.swap(i, small);
            small += 1;
        }
    }
    arr.swap(small, len - 1);

    let (left, rest) = arr.split_at_mut(small);
    hand_quicksort(left);
    // rest[0] is the pivot in its final position; sort everything after.
    hand_quicksort(&mut rest[1..]);
}

type SystemQS = QuickSort<Lomuto, FirstElement, InsertionSmallSort<LinearInsertion, 32>>;

fn percentile(times: &[u64], p: f64) -> u64 {
    let mut v = times.to_vec();
    v.sort_unstable();
    let idx = ((v.len() as f64 - 1.0) * p).round() as usize;
    v[idx]
}

fn mean_ns(times: &[u64]) -> f64 {
    times.iter().sum::<u64>() as f64 / times.len() as f64
}

fn stddev_ns(times: &[u64], mean: f64) -> f64 {
    let var = times
        .iter()
        .map(|&t| {
            let d = t as f64 - mean;
            d * d
        })
        .sum::<f64>()
        / times.len() as f64;
    var.sqrt()
}

fn fmt_ns(ns: f64) -> String {
    if ns < 1_000.0 {
        format!("{:>7.0} ns", ns)
    } else if ns < 1_000_000.0 {
        format!("{:>7.2} µs", ns / 1_000.0)
    } else if ns < 1_000_000_000.0 {
        format!("{:>7.2} ms", ns / 1_000_000.0)
    } else {
        format!("{:>7.2}  s", ns / 1_000_000_000.0)
    }
}

struct Stats {
    min: u64,
    median: u64,
    p90: u64,
    mean: f64,
    stddev: f64,
}

impl Stats {
    fn from(times: &[u64]) -> Self {
        let mean = mean_ns(times);
        Self {
            min: percentile(times, 0.0),
            median: percentile(times, 0.5),
            p90: percentile(times, 0.9),
            mean,
            stddev: stddev_ns(times, mean),
        }
    }

    fn print_row(&self, label: &str) {
        println!(
            "{:<10} {:>10} {:>10} {:>10} {:>10} {:>10}",
            label,
            fmt_ns(self.min as f64),
            fmt_ns(self.median as f64),
            fmt_ns(self.p90 as f64),
            fmt_ns(self.mean),
            fmt_ns(self.stddev),
        );
    }
}

/// Run `sort` on `arr` and return its wall-clock cost in nanoseconds.
fn time_sort(arr: &mut [u64], sort: impl FnOnce(&mut [u64])) -> u64 {
    let t = Instant::now();
    sort(arr);
    let dt = t.elapsed().as_nanos() as u64;
    debug_assert!(arr.is_sorted());
    dt
}

fn main() {
    let mut args = std::env::args().skip(1);
    let n: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(1_000_000);
    let runs: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(31);
    const WARMUP: usize = 3;

    // Seeded RNG so the input distribution is reproducible across runs.
    let mut rng = StdRng::seed_from_u64(0xC0FFEE);
    let shuffled = |rng: &mut StdRng| {
        let mut v: Vec<u64> = (0..n as u64).collect();
        v.shuffle(rng);
        v
    };
    let system_sort = |s: &mut [u64]| SystemQS::sort(s, &mut NoOpLogger);

    // Warmup: a few untimed runs of each to settle CPU frequency, prime
    // caches, and let the branch predictor lock in. Without this the first
    // timed iteration of whichever sort runs first is an outlier.
    for _ in 0..WARMUP {
        hand_quicksort(&mut shuffled(&mut rng));
        system_sort(&mut shuffled(&mut rng));
    }

    let mut hand_times = Vec::with_capacity(runs);
    let mut sys_times = Vec::with_capacity(runs);
    for iter in 0..runs {
        let mut a = shuffled(&mut rng);
        let mut b = a.clone();
        // Alternate which sort runs first: belt-and-braces guard against
        // any residual order bias the warmup didn't cover.
        let (th, ts) = if iter & 1 == 0 {
            let th = time_sort(&mut a, hand_quicksort);
            let ts = time_sort(&mut b, system_sort);
            (th, ts)
        } else {
            let ts = time_sort(&mut b, system_sort);
            let th = time_sort(&mut a, hand_quicksort);
            (th, ts)
        };
        hand_times.push(th);
        sys_times.push(ts);
    }

    let hand = Stats::from(&hand_times);
    let sys = Stats::from(&sys_times);

    println!("Quicksort comparison  N={n}  runs={runs}  warmup={WARMUP}");
    println!("    (u64, Lomuto, first pivot, insertion@<=32)");
    println!();
    println!("{:<10} {:>10} {:>10} {:>10} {:>10} {:>10}",
             "sort", "min", "median", "p90", "mean", "stddev");
    println!("{:-<10} {:->10} {:->10} {:->10} {:->10} {:->10}", "", "", "", "", "", "");
    hand.print_row("hand");
    sys.print_row("system");

    let ratio_med = sys.median as f64 / hand.median as f64;
    let ratio_mean = sys.mean / hand.mean;
    println!();
    println!("system / hand    median: {ratio_med:.3}×    mean: {ratio_mean:.3}×");
    if (ratio_med - 1.0).abs() < 0.05 {
        println!("→ within ±5% — abstraction has effectively zero cost");
    } else if ratio_med > 1.0 {
        println!("→ system is {:.1}% slower", (ratio_med - 1.0) * 100.0);
    } else {
        println!("→ system is {:.1}% faster", (1.0 - ratio_med) * 100.0);
    }
}
