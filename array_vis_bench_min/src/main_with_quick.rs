//! Minimal-build smoke test #2: also drag in QuickSort with one
//! concrete (partition, pivot) pair. Demonstrates that after the
//! `quick_partition_registry` extraction, picking QuickSort no longer
//! forces every other partition/pivot leaf — the only ones compiled
//! are `partition_lomuto` and `pivot_first`.

use std::time::Instant;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use partition_lomuto::LeftLeftPartition;
use pivot_first::FirstElement;
use quick_sort_lib::quick_sort::QuickSort;
use small_sort_basic::NoSmallSort;
use sort_logger::NoOpLogger;

fn shuffled(n: usize, seed: u64) -> Vec<u64> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut v: Vec<u64> = (0..n as u64).collect();
    v.shuffle(&mut rng);
    v
}

fn main() {
    println!("array_vis_bench_min_with_quick — LeftLeftPartition + FirstElement QuickSort, no wiring crate");

    let n = 100_000;
    let mut a = shuffled(n, 0xC0FFEE);
    let t = Instant::now();
    QuickSort::<LeftLeftPartition, FirstElement, NoSmallSort>::sort(&mut a, &mut NoOpLogger);
    println!("  QuickSort<LeftLeftPartition, First, NoSmall>  N={n}  {:>8.2?}", t.elapsed());
    assert!(a.is_sorted());
}
