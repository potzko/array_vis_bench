//! Performance regression gate for the per-crate refactor.
//!
//! Runs three hand-rolled u64 sorts against their `array_vis_bench`
//! equivalents (using `NoOpLogger` so the trait/inlining overhead is the
//! only thing being measured) and asserts each system sort stays within
//! a configurable margin of its hand baseline. Exits non-zero if any
//! pair regresses past the gate.
//!
//! Strategy matched across each pair so the comparison is fair:
//!
//! | sort      | partition / strategy           | small sort | other       |
//! |-----------|--------------------------------|------------|-------------|
//! | quick     | LeftLeftPartition + first pivot           | ins ≤32    | recursive   |
//! | merge     | top-down, copy-back            | ins ≤32    | aux Vec<T>  |
//! | heap      | binary max-forward (sift-down) | none       | iterative build |
//!
//! Usage:
//!   cargo run --release --bin compare_sorts
//!   cargo run --release --bin compare_sorts -- 1_000_000 31
//!   cargo run --release --bin compare_sorts -- 1_000_000 31 1.15   # override
//!
//! Build with `target-cpu=native` for fair numbers (the project's
//! `.cargo/config.toml` sets this by default).
//!
//! Single regression gate of **1.05** across all three pairs, calibrated
//! after the codegen-units=1 + `#[inline]` heapify fix landed. Measured
//! ratios: quick 0.95× (system faster), merge 0.93× (system faster),
//! heap 1.03× — all comfortably under the gate. If a future change pushes
//! any of them past 1.05, that's the signal that something cross-crate
//! stopped inlining or that the codegen-units split is back.
//!
//! Hand versions stay in their natural shape (tight loops) rather than
//! mirroring system-side choices, so the ratio reflects *real* framework
//! cost. If you tighten a hand implementation, expect its gate to move
//! with it.
//!
//! The third CLI arg overrides all three gates — useful for one-shot
//! loosening when a phase is known to regress temporarily.
//!
//! Exit codes:
//!   0 — every system sort within its gate
//!   1 — at least one pair regressed past its gate

use std::time::Instant;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use array_vis_bench_full::sorts::heap_sort::arity::Binary;
use array_vis_bench_full::sorts::heap_sort::arity_heap::ArityHeap;
use array_vis_bench_full::sorts::heap_sort::deep_heapify::Iterative;
use array_vis_bench_full::sorts::heap_sort::direction::MaxForward;
use array_vis_bench_full::sorts::heap_sort::heap_sort::NaryHeapSort;
use array_vis_bench_full::sorts::merge_sorts::top_down::TopDownMergeSort;
use array_vis_bench_full::sorts::quick_sorts::partitions::LeftLeftPartition;
use array_vis_bench_full::sorts::quick_sorts::pivot_selectors::FirstElement;
use array_vis_bench_full::sorts::quick_sorts::quick_sort::QuickSort;
use array_vis_bench_full::traits::log_traits::NoOpLogger;
use array_vis_bench_full::utils::small_sort::{InsertionSmallSort, LinearInsertion};

const SMALL_SORT_THRESHOLD: usize = 32;

// ── Hand-rolled baselines ────────────────────────────────────────────────────

/// Insertion sort the system path picks at len ≤ 32. Shared by quick and
/// merge so each pair's small-input regime is the same on both sides.
#[inline(always)]
fn hand_insertion_sort(arr: &mut [u64]) {
    for i in 1..arr.len() {
        let mut j = i;
        while j > 0 && arr[j] < arr[j - 1] {
            arr.swap(j, j - 1);
            j -= 1;
        }
    }
}

/// LeftLeftPartition + first-element pivot + insertion-sort under-32 threshold.
///
/// `#[inline(never)] #[no_mangle]` so the bin exposes a clean
/// `hand_quicksort` symbol — pair with the `asm_quick_sort_classic_*`
/// wrappers in `quick_sort_lib` and `cargo asm` to diff the two.
#[inline(never)]
#[no_mangle]
pub fn hand_quicksort(arr: &mut [u64]) {
    if arr.len() <= SMALL_SORT_THRESHOLD {
        hand_insertion_sort(arr);
        return;
    }

    // Move pivot to the end, scan forward, swap small-or-equal elements
    // to the front.
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
    hand_quicksort(&mut rest[1..]);
}

/// Top-down merge sort with insertion-sort cutoff and a single aux Vec.
/// Matches `TopDownMergeSort<InsertionSmallSort<LinearInsertion, 32>,
/// false, false>` (copy-back, no early-exit, no ping-pong).
#[inline(never)]
#[no_mangle]
pub fn hand_mergesort(arr: &mut [u64]) {
    if arr.len() < 2 {
        return;
    }
    let mut tmp = vec![0u64; arr.len()];
    hand_mergesort_rec(arr, &mut tmp);
}

fn hand_mergesort_rec(arr: &mut [u64], tmp: &mut [u64]) {
    let n = arr.len();
    if n <= SMALL_SORT_THRESHOLD {
        hand_insertion_sort(arr);
        return;
    }
    let mid = n / 2;
    {
        let (al, ar) = arr.split_at_mut(mid);
        let (tl, tr) = tmp.split_at_mut(mid);
        hand_mergesort_rec(al, tl);
        hand_mergesort_rec(ar, tr);
    }
    // Merge arr[..mid] and arr[mid..] into tmp, then copy back.
    let (al, ar) = arr.split_at(mid);
    let mut i = 0;
    let mut j = 0;
    let mut k = 0;
    while i < al.len() && j < ar.len() {
        if al[i] <= ar[j] {
            tmp[k] = al[i];
            i += 1;
        } else {
            tmp[k] = ar[j];
            j += 1;
        }
        k += 1;
    }
    while i < al.len() {
        tmp[k] = al[i];
        i += 1;
        k += 1;
    }
    while j < ar.len() {
        tmp[k] = ar[j];
        j += 1;
        k += 1;
    }
    arr.copy_from_slice(&tmp[..n]);
}

/// Binary max-heap sort (Floyd build + repeated sift-down extracts).
/// Matches `HeapSort<ArityHeap<Binary, MaxForward>, Iterative>`.
#[inline(never)]
#[no_mangle]
pub fn hand_heapsort(arr: &mut [u64]) {
    let n = arr.len();
    if n < 2 {
        return;
    }
    // Floyd build: sift down from the last internal node back to the root.
    for i in (0..n / 2).rev() {
        sift_down(arr, n, i);
    }
    // Extract loop: swap root → end, shrink heap, sift down new root.
    for end in (1..n).rev() {
        arr.swap(0, end);
        sift_down(arr, end, 0);
    }
}

/// Iterative sift-down — the natural tight loop. The system's matching
/// path goes `Iterative::deep_heapify` (iterative build) → `ArityHeap::
/// heapify` (recursive single-node sift-down). LLVM does TCO that inner
/// recursion most of the time, but not always; the leftover gap is what
/// the heap gate measures.
#[inline(always)]
fn sift_down(arr: &mut [u64], heap_size: usize, mut i: usize) {
    loop {
        let l = 2 * i + 1;
        let r = 2 * i + 2;
        let mut largest = i;
        if l < heap_size && arr[l] > arr[largest] {
            largest = l;
        }
        if r < heap_size && arr[r] > arr[largest] {
            largest = r;
        }
        if largest == i {
            return;
        }
        arr.swap(i, largest);
        i = largest;
    }
}

// ── System counterparts ──────────────────────────────────────────────────────

type SystemQS = QuickSort<LeftLeftPartition, FirstElement, InsertionSmallSort<LinearInsertion, 32>>;
type SystemMS = TopDownMergeSort<InsertionSmallSort<LinearInsertion, 32>, false, false>;
type SystemHS = NaryHeapSort<ArityHeap<Binary, MaxForward>, Iterative>;

// Stable `#[no_mangle]` wrappers around each system sort's u64 entry —
// the asm-side counterpart to `hand_quicksort` etc. `cargo asm` can pull
// these by name without guessing which generic monomorphisation to
// inspect; `#[inline(never)]` keeps the call boundary visible.
#[inline(never)]
#[no_mangle]
pub fn sys_quicksort_u64(arr: &mut [u64]) {
    SystemQS::sort(arr, &mut NoOpLogger);
}
#[inline(never)]
#[no_mangle]
pub fn sys_mergesort_u64(arr: &mut [u64]) {
    SystemMS::sort(arr, &mut NoOpLogger);
}
#[inline(never)]
#[no_mangle]
pub fn sys_heapsort_u64(arr: &mut [u64]) {
    SystemHS::sort(arr, &mut NoOpLogger);
}

// ── Stats ────────────────────────────────────────────────────────────────────

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
        format!("{:>8.0} ns", ns)
    } else if ns < 1_000_000.0 {
        format!("{:>8.2} µs", ns / 1_000.0)
    } else if ns < 1_000_000_000.0 {
        format!("{:>8.2} ms", ns / 1_000_000.0)
    } else {
        format!("{:>8.2}  s", ns / 1_000_000_000.0)
    }
}

#[derive(Clone, Copy)]
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
}

// ── Timed comparison ─────────────────────────────────────────────────────────

/// Run `sort` on `arr` and return its wall-clock cost in nanoseconds.
///
/// `&impl Fn(...)` (not `fn(...)`) so each caller monomorphises against
/// its concrete closure/function and the call inlines — function-pointer
/// indirection at this spot put quick-sort at ~1.09× system/hand even
/// when the underlying sort was at parity.
#[inline(always)]
fn time_sort(arr: &mut [u64], sort: &impl Fn(&mut [u64])) -> u64 {
    let t = Instant::now();
    sort(arr);
    let dt = t.elapsed().as_nanos() as u64;
    debug_assert!(arr.is_sorted());
    dt
}

/// One comparison line: hand vs system on the same input set, with
/// alternating run-order to flatten any residual ordering bias the
/// warmup didn't.
struct PairResult {
    label: &'static str,
    /// Maximum allowed `system_median / hand_median` ratio before this
    /// pair is reported as a regression. See module doc for per-sort
    /// rationale.
    gate: f64,
    hand: Stats,
    sys: Stats,
}

fn run_pair<H, S>(
    label: &'static str,
    gate: f64,
    n: usize,
    runs: usize,
    warmup: usize,
    rng: &mut StdRng,
    hand_sort: H,
    sys_sort: S,
) -> PairResult
where
    H: Fn(&mut [u64]),
    S: Fn(&mut [u64]),
{
    let shuffled = |rng: &mut StdRng| {
        let mut v: Vec<u64> = (0..n as u64).collect();
        v.shuffle(rng);
        v
    };

    for _ in 0..warmup {
        hand_sort(&mut shuffled(rng));
        sys_sort(&mut shuffled(rng));
    }

    let mut hand_times = Vec::with_capacity(runs);
    let mut sys_times = Vec::with_capacity(runs);
    for iter in 0..runs {
        let mut a = shuffled(rng);
        let mut b = a.clone();
        let (th, ts) = if iter & 1 == 0 {
            let th = time_sort(&mut a, &hand_sort);
            let ts = time_sort(&mut b, &sys_sort);
            (th, ts)
        } else {
            let ts = time_sort(&mut b, &sys_sort);
            let th = time_sort(&mut a, &hand_sort);
            (th, ts)
        };
        hand_times.push(th);
        sys_times.push(ts);
    }

    PairResult {
        label,
        gate,
        hand: Stats::from(&hand_times),
        sys: Stats::from(&sys_times),
    }
}

fn print_table(results: &[PairResult]) -> bool {
    println!(
        "{:<8} {:<8} {:>11} {:>11} {:>11} {:>11} {:>11}",
        "sort", "side", "min", "median", "p90", "mean", "stddev",
    );
    println!("{:-<8} {:-<8} {:->11} {:->11} {:->11} {:->11} {:->11}",
        "", "", "", "", "", "", "");

    let mut all_ok = true;
    for r in results {
        for (side, s) in [("hand", r.hand), ("system", r.sys)] {
            println!(
                "{:<8} {:<8} {:>11} {:>11} {:>11} {:>11} {:>11}",
                if side == "hand" { r.label } else { "" },
                side,
                fmt_ns(s.min as f64),
                fmt_ns(s.median as f64),
                fmt_ns(s.p90 as f64),
                fmt_ns(s.mean),
                fmt_ns(s.stddev),
            );
        }
        let ratio_med = r.sys.median as f64 / r.hand.median as f64;
        let ratio_mean = r.sys.mean / r.hand.mean;
        let status = if ratio_med <= r.gate {
            "OK"
        } else {
            all_ok = false;
            "FAIL"
        };
        println!(
            "         ratio (sys/hand)  median: {:>5.3}×  mean: {:>5.3}×  gate: {:>5.3}×  [{}]",
            ratio_med, ratio_mean, r.gate, status,
        );
        println!();
    }
    all_ok
}

// Uniform 1.05 gate across all three sorts — see module doc for why.
const QUICK_GATE: f64 = 1.05;
const MERGE_GATE: f64 = 1.05;
const HEAP_GATE: f64 = 1.05;

fn main() {
    let mut args = std::env::args().skip(1);
    let n: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(1_000_000);
    let runs: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(31);
    // Optional global override: a single ratio applied to every sort.
    // Useful for one-shot loosening when you expect a known regression
    // during in-flight refactor work; tightening past the per-sort
    // defaults is also fine.
    let global_gate: Option<f64> = args.next().and_then(|s| s.parse().ok());
    const WARMUP: usize = 3;

    let mut rng = StdRng::seed_from_u64(0xC0FFEE);

    let pairs = [
        run_pair(
            "quick",
            global_gate.unwrap_or(QUICK_GATE),
            n,
            runs,
            WARMUP,
            &mut rng,
            hand_quicksort,
            sys_quicksort_u64,
        ),
        run_pair(
            "merge",
            global_gate.unwrap_or(MERGE_GATE),
            n,
            runs,
            WARMUP,
            &mut rng,
            hand_mergesort,
            sys_mergesort_u64,
        ),
        run_pair(
            "heap",
            global_gate.unwrap_or(HEAP_GATE),
            n,
            runs,
            WARMUP,
            &mut rng,
            hand_heapsort,
            sys_heapsort_u64,
        ),
    ];

    let gate_str = match global_gate {
        Some(g) => format!("global override ±{:.0}%", (g - 1.0) * 100.0),
        None => format!("uniform default ±{:.0}%", (QUICK_GATE - 1.0) * 100.0),
    };
    println!("Sort comparison  N={n}  runs={runs}  warmup={WARMUP}  gates: {gate_str}");
    println!("    u64, shuffled permutation [0, N)");
    println!();
    let all_ok = print_table(&pairs);

    if !all_ok {
        eprintln!("✗ at least one system sort regressed past its gate");
        std::process::exit(1);
    }
    println!("✓ all system sorts within gate");
}
