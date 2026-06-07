//! Faithful stub of the runtime ABI the generated code targets. Names and
//! signatures mirror the REAL crates so emitted `AlgorithmEntry` blocks port
//! with only path edits:
//!   - `avb_abi::SortLogger`        ↔ `sort_logger::SortLogger`
//!   - `avb_abi::SortAlgo`          ↔ `array_vis_bench_traits::SortAlgo`
//!   - `avb_abi::{HasTimeBounds,…}` ↔ `array_vis_bench_traits::{HasTimeBounds,…}`
//!   - `avb_abi::Complexity`        ↔ `array_vis_bench_traits::Complexity`
//!   - `avb_abi::{AlgorithmEntry, Category, RunConfig, ALGORITHMS,
//!      run_sort_with_input, register_sort_variant}` ↔ `array_vis_bench_core::bench_registry::*`
//!
//! Deliberately a small-but-representative subset: just enough surface for the
//! emit backend to generate against and for a test to drive a sort through a
//! `&mut dyn SortLogger` and observe a real event log.

use std::sync::Mutex;

use linkme::distributed_slice;

// ── the event ABI sorts are written against (dyn-compatible: every method is
//    non-generic beyond the trait's `T`) ──────────────────────────────────────
pub trait SortLogger<T> {
    fn create_array(&mut self, len: usize);
    fn write(&mut self, index: usize, value: T);
    fn compare(&mut self, i: usize, ii: usize);
    fn swap(&mut self, i: usize, ii: usize);
}

/// The perf-path logger: every call compiles to nothing.
pub struct NoOpLogger;
impl<T> SortLogger<T> for NoOpLogger {
    fn create_array(&mut self, _len: usize) {}
    fn write(&mut self, _index: usize, _value: T) {}
    fn compare(&mut self, _i: usize, _ii: usize) {}
    fn swap(&mut self, _i: usize, _ii: usize) {}
}

/// The visualiser-side logger (stands in for `VisualizerLogger`): records the
/// event stream so a test can assert "the program actually drove the logger".
#[derive(Default)]
pub struct CaptureLogger {
    pub events: Vec<String>,
}
impl SortLogger<usize> for CaptureLogger {
    fn create_array(&mut self, len: usize) {
        self.events.push(format!("create {len}"));
    }
    fn write(&mut self, index: usize, value: usize) {
        self.events.push(format!("write {index} {value}"));
    }
    fn compare(&mut self, i: usize, ii: usize) {
        self.events.push(format!("cmp {i} {ii}"));
    }
    fn swap(&mut self, i: usize, ii: usize) {
        self.events.push(format!("swap {i} {ii}"));
    }
}

// ── the type-level annotation traits the AlgorithmEntry complexity fields
//    inherit from (`<Ty as HasTimeBounds>::WORST`, etc.) ─────────────────────
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Complexity(pub &'static str);
impl Complexity {
    pub const UNKNOWN: Complexity = Complexity("O(?)");
    pub const fn from_str(s: &'static str) -> Complexity {
        Complexity(s)
    }
}

pub trait HasTimeBounds {
    const WORST: Complexity = Complexity::UNKNOWN;
    const BEST: Complexity = Self::WORST;
    const AVERAGE: Complexity = Self::WORST;
}
pub trait HasSpace {
    const SPACE: Complexity = Complexity::UNKNOWN;
}
pub trait HasStability {
    const STABLE: bool = false;
}

/// The composition contract every leaf is written against. `U: ?Sized` so the
/// same impl serves both a concrete logger (perf path) and `dyn SortLogger`
/// (visualiser path) — mirrors the real `U: ?Sized + SortLogger<T>` pattern.
pub trait SortAlgo<T: Ord + Copy, U: ?Sized + SortLogger<T>> {
    fn sort(arr: &mut [T], logger: &mut U);
}

// ── the registry record the compiler EMITS (the de-hollowed center) ──────────
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Sort,
    Rotation,
    Partition,
    Merge,
    SmallSort,
    QuickSelect,
}

pub struct RunConfig {
    pub size: usize,
    pub seed: u64,
}

pub struct AlgorithmEntry {
    pub name: &'static str,
    pub category: Category,
    pub worst: Complexity,
    pub best: Complexity,
    pub average: Complexity,
    pub space: Complexity,
    pub stable: bool,
    pub adaptive: bool,
    pub max_input_size: Option<usize>,
    pub run_with_input: fn(input_name: &str, config: &RunConfig, &mut dyn SortLogger<usize>),
    pub run_correctness: fn(),
}

#[distributed_slice]
pub static ALGORITHMS: [AlgorithmEntry] = [..];

/// Sort driver shared by every emitted `Category::Sort` entry: build the named
/// input, then run the algorithm on it, emitting every event on `logger`. (The
/// stub ignores `input_name` and synthesises from `config`; the real one looks
/// the input up in `SORT_INPUTS`.)
pub fn run_sort_with_input(
    _input_name: &str,
    config: &RunConfig,
    sort_dyn: fn(&mut [usize], &mut dyn SortLogger<usize>),
    logger: &mut dyn SortLogger<usize>,
) {
    let n = config.size.max(1);
    let mut arr: Vec<usize> = (0..config.size)
        .map(|i| (i.wrapping_mul(2_654_435_761) ^ config.seed as usize) % n)
        .collect();
    sort_dyn(&mut arr, logger);
}

/// Correctness battery for `Category::Sort` (stub of `correctness::sort_battery`):
/// runs a few inputs through `NoOpLogger` and asserts the result is sorted.
pub fn assert_sorts(sort_dyn: fn(&mut [usize], &mut dyn SortLogger<usize>)) {
    for &n in &[0usize, 1, 2, 7, 33, 128] {
        let mut arr: Vec<usize> = (0..n).rev().collect();
        sort_dyn(&mut arr, &mut NoOpLogger);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]), "not sorted at n={n}");
    }
}

// ── the non-Sort category contracts ──────────────────────────────────────────
// Each mirrors `SortAlgo` in spirit (a single static entry point taking the
// array + the op's parameter + a `&mut dyn SortLogger`), but with the shape the
// category actually needs. The three emit drivers
// (`spec_core::emit_drivers::{partition,merge,rotation}`) generate code against
// exactly these signatures.

/// `Category::Partition` contract. Partitions `arr` around the value at
/// `pivot_index`, emitting the scan, and returns the pivot's final index.
pub trait Partitioner {
    fn partition(arr: &mut [usize], pivot_index: usize, logger: &mut dyn SortLogger<usize>) -> usize;
}

/// `Category::Merge` contract. `arr[..mid]` and `arr[mid..]` are each already
/// sorted; merges them in place, emitting the merge.
pub trait Merger {
    fn merge(arr: &mut [usize], mid: usize, logger: &mut dyn SortLogger<usize>);
}

/// `Category::Rotation` contract. Left-rotates `arr` by `mid` (swaps the two
/// blocks `arr[..mid]` / `arr[mid..]`), emitting the moves.
pub trait Rotator {
    fn rotate(arr: &mut [usize], mid: usize, logger: &mut dyn SortLogger<usize>);
}

/// Build a partition input (array + a mid-ish pivot index) and run the op,
/// emitting every event on `logger`. Mirrors `run_sort_with_input`'s shape.
pub fn run_partition_with_input(
    _input_name: &str,
    config: &RunConfig,
    partition_fn: fn(&mut [usize], usize, &mut dyn SortLogger<usize>) -> usize,
    logger: &mut dyn SortLogger<usize>,
) {
    let n = config.size.max(1);
    let mut arr: Vec<usize> = (0..config.size)
        .map(|i| (i.wrapping_mul(2_654_435_761) ^ config.seed as usize) % n)
        .collect();
    let pivot_index = arr.len() / 2;
    partition_fn(&mut arr, pivot_index, logger);
}

/// Build two sorted halves and merge them, emitting the merge on `logger`.
pub fn run_merge_with_input(
    _input_name: &str,
    config: &RunConfig,
    merge_fn: fn(&mut [usize], usize, &mut dyn SortLogger<usize>),
    logger: &mut dyn SortLogger<usize>,
) {
    let n = config.size.max(1);
    let mut arr: Vec<usize> = (0..config.size)
        .map(|i| (i.wrapping_mul(2_654_435_761) ^ config.seed as usize) % n)
        .collect();
    let mid = arr.len() / 2;
    arr[..mid].sort_unstable();
    arr[mid..].sort_unstable();
    merge_fn(&mut arr, mid, logger);
}

/// Build an input and rotate it, emitting the moves on `logger`.
pub fn run_rotation_with_input(
    _input_name: &str,
    config: &RunConfig,
    rotate_fn: fn(&mut [usize], usize, &mut dyn SortLogger<usize>),
    logger: &mut dyn SortLogger<usize>,
) {
    let n = config.size.max(1);
    let mut arr: Vec<usize> = (0..config.size)
        .map(|i| (i.wrapping_mul(2_654_435_761) ^ config.seed as usize) % n)
        .collect();
    let mid = arr.len() / 3;
    rotate_fn(&mut arr, mid, logger);
}

/// Correctness battery for `Category::Partition`: after partitioning around the
/// value at the pivot index, everything left of the returned index is ≤ pivot
/// and everything right is ≥ pivot.
pub fn assert_partitions(partition_fn: fn(&mut [usize], usize, &mut dyn SortLogger<usize>) -> usize) {
    for &n in &[1usize, 2, 7, 33, 128] {
        let mut arr: Vec<usize> = (0..n).rev().collect();
        let pivot_index = n / 2;
        let p = partition_fn(&mut arr, pivot_index, &mut NoOpLogger);
        assert!(p < n, "pivot index {p} out of bounds at n={n}");
        let pivot = arr[p];
        assert!(arr[..p].iter().all(|&x| x <= pivot), "left of pivot not ≤ pivot at n={n}");
        assert!(arr[p + 1..].iter().all(|&x| x >= pivot), "right of pivot not ≥ pivot at n={n}");
    }
}

/// Correctness battery for `Category::Merge`: merging two sorted halves yields a
/// fully sorted permutation of the inputs.
pub fn assert_merges(merge_fn: fn(&mut [usize], usize, &mut dyn SortLogger<usize>)) {
    for &n in &[0usize, 1, 2, 7, 33, 128] {
        let mut arr: Vec<usize> = (0..n).map(|i| (i * 7) % n.max(1)).collect();
        let mid = n / 2;
        arr[..mid].sort_unstable();
        arr[mid..].sort_unstable();
        let mut expected = arr.clone();
        expected.sort_unstable();
        merge_fn(&mut arr, mid, &mut NoOpLogger);
        assert_eq!(arr, expected, "merge did not produce a sorted permutation at n={n}");
    }
}

/// Correctness battery for `Category::Rotation`: rotating by `mid` equals the
/// reference `rotate_left(mid)`.
pub fn assert_rotations(rotate_fn: fn(&mut [usize], usize, &mut dyn SortLogger<usize>)) {
    for &n in &[0usize, 1, 2, 7, 33, 128] {
        for &mid in &[0usize, 1, n / 3, n / 2, n] {
            let mut arr: Vec<usize> = (0..n).collect();
            let mut expected = arr.clone();
            if n > 0 {
                expected.rotate_left(mid % n);
            }
            rotate_fn(&mut arr, mid, &mut NoOpLogger);
            assert_eq!(arr, expected, "rotation by {mid} wrong at n={n}");
        }
    }
}

/// The menu-tree side effect (stub of `register_sort_variant`): records the
/// path so a test could inspect the picker tree. Faithful shape, minimal body.
static MENU: Mutex<Vec<String>> = Mutex::new(Vec::new());
pub fn register_sort_variant(name: &str, path: &[&str], _facets: &[(&str, &str)]) {
    MENU.lock().unwrap().push(format!("{}/{name}", path.join("/")));
}
pub fn menu_paths() -> Vec<String> {
    MENU.lock().unwrap().clone()
}
