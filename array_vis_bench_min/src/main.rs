//! Minimal-build smoke test. Pulls in just two algorithm leaves
//! (`heap_sort_lib` + `insertion_sort_lib`) and runs each on a small
//! random u64 array, printing the timing. No codegen, no wiring crate,
//! no menu tree, no GIF/MP4 visualiser — only the trait + logger
//! crates that any algorithm leaf must depend on.
//!
//! Run with `cargo run -p array_vis_bench_min --release`.

use std::time::Instant;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use heap_sort_lib::arity::Binary;
use heap_sort_lib::arity_heap::ArityHeap;
use heap_sort_lib::deep_heapify::Iterative;
use heap_sort_lib::direction::MaxForward;
use heap_sort_lib::heap_sort::HeapSort;
use insertion_sort_lib::InsertionSort;
use small_sort_insertion_strategy::LinearInsertion;
use sort_logger::NoOpLogger;

fn shuffled(n: usize, seed: u64) -> Vec<u64> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut v: Vec<u64> = (0..n as u64).collect();
    v.shuffle(&mut rng);
    v
}

fn time<F: FnOnce()>(label: &str, n: usize, f: F) {
    let t = Instant::now();
    f();
    let dt = t.elapsed();
    println!("  {label:<24} N={n}  {:>8.2?}", dt);
}

fn main() {
    println!("array_vis_bench_min — two sorts, no wiring crate");
    println!();

    let small_n = 1_000;
    let large_n = 100_000;

    {
        let mut a = shuffled(small_n, 0xC0FFEE);
        time("InsertionSort<Linear>", small_n, || {
            InsertionSort::<LinearInsertion>::sort(&mut a, &mut NoOpLogger);
        });
        assert!(a.is_sorted());
    }

    {
        let mut a = shuffled(large_n, 0xC0FFEE);
        time("HeapSort<Binary>", large_n, || {
            HeapSort::<ArityHeap<Binary, MaxForward>, Iterative>::sort(&mut a, &mut NoOpLogger);
        });
        assert!(a.is_sorted());
    }

    println!();
    println!("done.");
}
