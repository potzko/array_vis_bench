//! A/B: the current `PartitionScheme` (tuple return) vs the prototype
//! `PartitionSchemeV` (visitor pattern, generalised over pivot arity).
//!
//! Single-pivot side: drives both traits through the same recursive
//! quicksort skeleton, on the same input. The only difference is the
//! partition→sort handshake.
//!
//! Dual-pivot side: demonstrates that the unified `PartitionSchemeV`
//! (with `N_PIVOTS = 2`) handles Yaroslavskiy's partition cleanly, and
//! that the same `qs_visitor` recursion driver works for it without
//! per-arity duplication. Compared against the current
//! `DualPivotQuickSort` from `quick_sort_lib`.
//!
//! Run with `cargo run --release --bin partition_perf_compare`.

use std::ops::Range;
use std::time::Instant;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use array_vis_bench_full::sorts::quick_sorts::dual_pivot_quick_sort::DualPivotQuickSort;
use array_vis_bench_full::sorts::quick_sorts::partitions::{
    Block, Hoare, Lomuto, MovingPivot, ThreeWay,
};
use array_vis_bench_full::sorts::quick_sorts::pivot_selectors::{
    CombinedSelector, FirstElement, MiddleElement,
};
use array_vis_bench_full::traits::log_traits::NoOpLogger;
use array_vis_bench_full::utils::small_sort::{InsertionSmallSort, LinearInsertion};
use array_vis_bench_traits::{
    Complexity, DualPivotSelector, HasSpace, HasStability, HasTimeBounds, PartitionScheme,
    PartitionSchemeV, PartitionVisitor, PivotSelector,
};
use sort_logger::SortLogger;

const SMALL_SORT_THRESHOLD: usize = 32;

// ── Shared small sort (insertion ≤ 32) ──────────────────────────────────────

#[inline(always)]
fn small_insertion_sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for i in 1..arr.len() {
        let mut j = i;
        while j > 0 && logger.cmp_lt(arr, j, j - 1) {
            logger.swap(arr, j, j - 1);
            j -= 1;
        }
    }
}

// ── Single-pivot A: tuple-return path ───────────────────────────────────────

fn qs_tuple<T, U, P, V>(arr: &mut [T], logger: &mut U)
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    P: PartitionScheme,
    V: PivotSelector,
{
    if arr.len() <= SMALL_SORT_THRESHOLD {
        small_insertion_sort(arr, logger);
        return;
    }
    let pivot = V::select(arr, logger);
    let (left_end, right_start) = P::partition(arr, logger, pivot);
    qs_tuple::<T, U, P, V>(&mut arr[..left_end], logger);
    qs_tuple::<T, U, P, V>(&mut arr[right_start..], logger);
}

// ── Visitor B: unified driver (single- and dual-pivot via const N_PIVOTS) ──
//
// Collect up to 4 ranges on the stack — covers 2-way (2 calls),
// 3-way (2 calls), dual-pivot Yaroslavskiy (3 calls), and a
// hypothetical 5-region eq-pinning dual-pivot (3 calls). Each
// `unsorted` call inlines into a single index write + length bump.

struct CollectVisitor {
    ranges: [Range<usize>; 4],
    n: u8,
}

impl PartitionVisitor for CollectVisitor {
    #[inline(always)]
    fn unsorted(&mut self, r: Range<usize>) {
        unsafe { *self.ranges.get_unchecked_mut(self.n as usize) = r };
        self.n += 1;
    }
}

/// Generic over pivot arity: picks `P::N_PIVOTS` pivots via the
/// supplied [`PivotInput`], partitions, recurses on each emitted range.
trait PivotInput {
    const N: usize;
    fn pick<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
        out: &mut [usize; 2],
    );
}

/// Single-pivot adapter: forwards to a `PivotSelector`.
struct OnePivot<V: PivotSelector>(std::marker::PhantomData<V>);
impl<V: PivotSelector> PivotInput for OnePivot<V> {
    const N: usize = 1;
    #[inline(always)]
    fn pick<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
        out: &mut [usize; 2],
    ) {
        out[0] = V::select(arr, logger);
    }
}

/// Dual-pivot adapter: forwards to a `DualPivotSelector`.
struct TwoPivot<D: DualPivotSelector>(std::marker::PhantomData<D>);
impl<D: DualPivotSelector> PivotInput for TwoPivot<D> {
    const N: usize = 2;
    #[inline(always)]
    fn pick<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
        out: &mut [usize; 2],
    ) {
        let (p1, p2) = D::select(arr, logger);
        out[0] = p1;
        out[1] = p2;
    }
}

fn qs_visitor<T, U, P, V>(arr: &mut [T], logger: &mut U)
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    P: PartitionSchemeV,
    V: PivotInput,
{
    debug_assert_eq!(P::N_PIVOTS, V::N);
    if arr.len() <= SMALL_SORT_THRESHOLD {
        small_insertion_sort(arr, logger);
        return;
    }
    let mut pivots = [0usize; 2];
    V::pick(arr, logger, &mut pivots);
    let mut visitor = CollectVisitor { ranges: [0..0, 0..0, 0..0, 0..0], n: 0 };
    P::partition::<T, U, _>(arr, logger, &pivots[..V::N], &mut visitor);
    let n = visitor.n as usize;
    let mut i = 0;
    while i < n {
        let r = visitor.ranges[i].clone();
        qs_visitor::<T, U, P, V>(&mut arr[r], logger);
        i += 1;
    }
}

// ── Yaroslavskiy dual-pivot (PartitionSchemeV, N_PIVOTS = 2) ────────────────
//
// Mirrors the body of `dual_pivot_quick_sort::dual_pivot_recursive`
// but emits the 3 unsorted regions through the visitor. Same pivot
// placement (p1 → index 0, p2 → index last), same scan loop.

pub struct Yaroslavskiy;

impl PartitionSchemeV for Yaroslavskiy {
    const NAME: &'static str = "yaroslavskiy";
    const N_PIVOTS: usize = 2;

    #[inline]
    fn partition<T, U, V>(
        arr: &mut [T],
        logger: &mut U,
        pivots: &[usize],
        visitor: &mut V,
    ) where
        T: Ord + Copy,
        U: ?Sized + SortLogger<T>,
        V: PartitionVisitor,
    {
        let len = arr.len();
        if len < 2 {
            return;
        }
        let last = len - 1;
        let (p1_idx, p2_idx_raw) = (pivots[0], pivots[1]);

        // Place pivots at the ends, mirroring dual_pivot_quick_sort.rs.
        let p2_idx = {
            logger.swap(arr, p1_idx, 0);
            if p2_idx_raw == p1_idx { 0 }
            else if p2_idx_raw == 0 { p1_idx }
            else { p2_idx_raw }
        };
        logger.swap(arr, p2_idx, last);
        if logger.cmp_gt(arr, 0, last) {
            logger.swap(arr, 0, last);
        }

        let p1 = arr[0];
        let p2 = arr[last];

        let mut lt = 1;
        let mut i = 1;
        let mut gt = last - 1;

        while i <= gt {
            if logger.cmp_lt_data(arr, i, p1) {
                logger.swap(arr, i, lt);
                lt += 1;
                i += 1;
            } else if logger.cmp_gt_data(arr, i, p2) {
                while i < gt && logger.cmp_gt_data(arr, gt, p2) {
                    gt -= 1;
                }
                logger.swap(arr, i, gt);
                if gt == 0 {
                    break;
                }
                gt -= 1;
                if logger.cmp_lt_data(arr, i, p1) {
                    logger.swap(arr, i, lt);
                    lt += 1;
                }
                i += 1;
            } else {
                i += 1;
            }
        }

        lt -= 1;
        gt += 1;
        logger.swap(arr, 0, lt);
        logger.swap(arr, last, gt);

        // Three unsorted regions; pivots at lt and gt are placed.
        visitor.unsorted(0..lt);
        if lt + 1 < gt {
            visitor.unsorted(lt + 1..gt);
        }
        if gt + 1 < len {
            visitor.unsorted(gt + 1..len);
        }
    }
}
// Bare composable annotations so anyone composing on Yaroslavskiy
// doesn't need to special-case it. Worst O(N) per partition step; the
// quicksort wrapper aggregates over log N depth.
impl HasTimeBounds for Yaroslavskiy {
    const WORST: Complexity = Complexity::N1;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N1;
}
impl HasSpace for Yaroslavskiy {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for Yaroslavskiy {
    const STABLE: bool = false;
}

// ── Timing harness ──────────────────────────────────────────────────────────

#[inline(always)]
fn time_sort(arr: &mut [u64], sort: &impl Fn(&mut [u64])) -> u64 {
    let t = Instant::now();
    sort(arr);
    let dt = t.elapsed().as_nanos() as u64;
    debug_assert!(arr.is_sorted());
    dt
}

fn percentile(times: &[u64], p: f64) -> u64 {
    let mut v = times.to_vec();
    v.sort_unstable();
    let idx = ((v.len() as f64 - 1.0) * p).round() as usize;
    v[idx]
}

fn fmt_ns(ns: f64) -> String {
    if ns < 1_000.0 {
        format!("{:>7.0} ns", ns)
    } else if ns < 1_000_000.0 {
        format!("{:>7.2} µs", ns / 1_000.0)
    } else {
        format!("{:>7.2} ms", ns / 1_000_000.0)
    }
}

struct PairResult {
    label: &'static str,
    tuple: Vec<u64>,
    visitor: Vec<u64>,
}

fn run_pair(
    label: &'static str,
    n: usize,
    runs: usize,
    warmup: usize,
    rng: &mut StdRng,
    tuple_fn: fn(&mut [u64]),
    visitor_fn: fn(&mut [u64]),
) -> PairResult {
    let shuffled = |rng: &mut StdRng| {
        let mut v: Vec<u64> = (0..n as u64).collect();
        v.shuffle(rng);
        v
    };
    for _ in 0..warmup {
        tuple_fn(&mut shuffled(rng));
        visitor_fn(&mut shuffled(rng));
    }
    let mut tuple = Vec::with_capacity(runs);
    let mut visitor = Vec::with_capacity(runs);
    for iter in 0..runs {
        let mut a = shuffled(rng);
        let mut b = a.clone();
        if iter & 1 == 0 {
            tuple.push(time_sort(&mut a, &tuple_fn));
            visitor.push(time_sort(&mut b, &visitor_fn));
        } else {
            visitor.push(time_sort(&mut b, &visitor_fn));
            tuple.push(time_sort(&mut a, &tuple_fn));
        }
    }
    PairResult { label, tuple, visitor }
}

fn report(r: &PairResult) {
    let t_min = percentile(&r.tuple, 0.0) as f64;
    let t_med = percentile(&r.tuple, 0.5) as f64;
    let v_min = percentile(&r.visitor, 0.0) as f64;
    let v_med = percentile(&r.visitor, 0.5) as f64;
    let t_mean = r.tuple.iter().sum::<u64>() as f64 / r.tuple.len() as f64;
    let v_mean = r.visitor.iter().sum::<u64>() as f64 / r.visitor.len() as f64;
    println!(
        "  {:<22}  tuple min {} med {}   visitor min {} med {}   ratio med {:.3}× mean {:.3}×",
        r.label,
        fmt_ns(t_min),
        fmt_ns(t_med),
        fmt_ns(v_min),
        fmt_ns(v_med),
        v_med / t_med,
        v_mean / t_mean,
    );
}

// ── Per-variant entry points (one bin symbol each so layout stays stable) ──

type SS = InsertionSmallSort<LinearInsertion, 32>;

macro_rules! variant {
    ($tuple_name:ident, $vis_name:ident, $part:ty, $pivot:ty) => {
        #[inline(never)]
        #[no_mangle]
        pub fn $tuple_name(arr: &mut [u64]) {
            qs_tuple::<u64, NoOpLogger, $part, $pivot>(arr, &mut NoOpLogger);
        }
        #[inline(never)]
        #[no_mangle]
        pub fn $vis_name(arr: &mut [u64]) {
            qs_visitor::<u64, NoOpLogger, $part, OnePivot<$pivot>>(arr, &mut NoOpLogger);
        }
    };
}

variant!(qs_tuple_lomuto,    qs_visitor_lomuto,    Lomuto,      FirstElement);
variant!(qs_tuple_hoare,     qs_visitor_hoare,     Hoare,       FirstElement);
variant!(qs_tuple_threeway,  qs_visitor_threeway,  ThreeWay,    FirstElement);
variant!(qs_tuple_movingpiv, qs_visitor_movingpiv, MovingPivot, FirstElement);
variant!(qs_tuple_block,     qs_visitor_block,     Block,       FirstElement);

// Dual-pivot: tuple side uses the existing DualPivotQuickSort algorithm;
// visitor side runs Yaroslavskiy as a `PartitionSchemeV` through the
// same `qs_visitor` driver as the single-pivot variants — proves the
// unification compiles and lowers identically.

type DPS = CombinedSelector<FirstElement, MiddleElement>;

#[inline(never)]
#[no_mangle]
pub fn qs_tuple_dualpiv(arr: &mut [u64]) {
    DualPivotQuickSort::<DPS, SS>::sort(arr, &mut NoOpLogger);
}
#[inline(never)]
#[no_mangle]
pub fn qs_visitor_dualpiv(arr: &mut [u64]) {
    qs_visitor::<u64, NoOpLogger, Yaroslavskiy, TwoPivot<DPS>>(arr, &mut NoOpLogger);
}

fn main() {
    let n: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1_000_000);
    let runs: usize = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(31);
    const WARMUP: usize = 3;
    let mut rng = StdRng::seed_from_u64(0xC0FFEE);

    println!("Partition trait A/B  N={n}  runs={runs}  warmup={WARMUP}");
    println!("  u64, shuffled permutation [0, N)\n");
    println!("  single-pivot: FirstElement, insertion ≤ {SMALL_SORT_THRESHOLD}\n");

    let pairs = [
        run_pair("lomuto",        n, runs, WARMUP, &mut rng, qs_tuple_lomuto,    qs_visitor_lomuto),
        run_pair("hoare",         n, runs, WARMUP, &mut rng, qs_tuple_hoare,     qs_visitor_hoare),
        run_pair("three-way",     n, runs, WARMUP, &mut rng, qs_tuple_threeway,  qs_visitor_threeway),
        run_pair("moving-pivot",  n, runs, WARMUP, &mut rng, qs_tuple_movingpiv, qs_visitor_movingpiv),
        run_pair("block",         n, runs, WARMUP, &mut rng, qs_tuple_block,     qs_visitor_block),
    ];
    for p in &pairs { report(p); }

    println!();
    println!("  dual-pivot (Yaroslavskiy, CombinedSelector<First, Middle>):");
    let dp = run_pair("dual-pivot", n, runs, WARMUP, &mut rng, qs_tuple_dualpiv, qs_visitor_dualpiv);
    report(&dp);
}
