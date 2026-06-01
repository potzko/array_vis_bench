//! Correctness check for the shared heap-extract core.
//!
//! Exercises both call sites with the same shuffled inputs:
//!   - [`QuickHeapSort`] — confirms the shared-core refactor didn't break
//!     the bespoke recursion-rebuild driver.
//!   - [`QuickSort<HeapExtract<…>, NoPivot, …>`] — confirms the new
//!     [`PartitionScheme`] sorts under the generic QuickSort driver.

use array_vis_bench_traits::NoPivot;
use heap_sort_lib::arity::{Binary, Ternary};
use heap_sort_lib::deep_heapify::{Iterative, Recursive};
use heap_sort_lib::set_quick_select::SequentialSet;
use quick_heap_sort_lib::heap_pair::AryPair;
use quick_heap_sort_lib::{heap_extract::HeapExtract, quick_heap_sort::QuickHeapSort};
use quick_select_lib::RecursiveQuickSelect;
use quick_sort_lib::quick_sort::QuickSort;
use small_sort_basic::NoSmallSort;
use sort_logger::NoOpLogger;

/// Pseudo-Fisher–Yates shuffle, seeded LCG — no rand dep, deterministic.
fn shuffled(n: usize, seed: u64) -> Vec<u64> {
    let mut v: Vec<u64> = (0..n as u64).collect();
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for i in (1..n).rev() {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (state as usize) % (i + 1);
        v.swap(i, j);
    }
    v
}

fn check_sorts<F: Fn(&mut [u64], &mut NoOpLogger)>(label: &str, sort: F) {
    for &n in &[0usize, 1, 2, 3, 4, 7, 16, 17, 50, 100, 500, 1000] {
        for seed in 0..3u64 {
            let mut a = shuffled(n, seed);
            sort(&mut a, &mut NoOpLogger);
            assert!(
                a.windows(2).all(|w| w[0] <= w[1]),
                "{label}: n={n} seed={seed} unsorted: {a:?}",
            );
            for (i, &v) in a.iter().enumerate() {
                assert_eq!(v, i as u64, "{label}: n={n} seed={seed} missing element");
            }
        }
    }
}

#[test]
fn quick_heap_sort_still_sorts_after_shared_core_refactor() {
    check_sorts(
        "QuickHeapSort<Binary, Iterative, NoSmallSort>",
        |a, l| QuickHeapSort::<Binary, Iterative, NoSmallSort>::sort(a, l),
    );
    check_sorts(
        "QuickHeapSort<Ternary, Recursive, NoSmallSort>",
        |a, l| QuickHeapSort::<Ternary, Recursive, NoSmallSort>::sort(a, l),
    );
}

#[test]
fn quick_sort_over_heap_extract_sorts() {
    check_sorts(
        "QuickSort<HeapExtract<AryPair<Binary>, Iterative>, NoPivot, NoSmallSort>",
        |a, l| {
            QuickSort::<HeapExtract<AryPair<Binary>, Iterative>, NoPivot, NoSmallSort>::sort(
                a, l,
            )
        },
    );
    check_sorts(
        "QuickSort<HeapExtract<AryPair<Ternary>, Recursive>, NoPivot, NoSmallSort>",
        |a, l| {
            QuickSort::<HeapExtract<AryPair<Ternary>, Recursive>, NoPivot, NoSmallSort>::sort(
                a, l,
            )
        },
    );
}

/// Bespoke `QuickHeapSort` with a SetQuickSelect-driven build that recurses
/// through `HeapExtract`. Replaces the old `PartitionDrivenDeepHeapify`
/// test now that the cycle closes via `SequentialSet<RecursiveQuickSelect<HeapExtract<...>, NoPivot>>`.
/// The head-count rule still bounds the inner HeapExtract's DH to a leaf.
#[test]
fn quick_heap_sort_with_set_quick_select_build_sorts() {
    type InnerHE = HeapExtract<AryPair<Binary>, Iterative>;
    type SQS = SequentialSet<RecursiveQuickSelect<InnerHE, NoPivot>>;
    // Use `Size2SmallSort` (THRESHOLD=2) so the bespoke QHS small-sort
    // gate triggers on len ≤ 2; NoSmallSort is also fine, but Size2
    // exercises the SS branch too.
    type Sort = QuickHeapSort<Binary, SQS, small_sort_basic::Size2SmallSort>;

    for &n in &[0usize, 1, 2, 3, 7, 16, 50, 128] {
        for seed in 0..2u64 {
            let mut a = shuffled(n, seed);
            Sort::sort(&mut a, &mut NoOpLogger);
            assert!(
                a.windows(2).all(|w| w[0] <= w[1]),
                "QHS+SQS: n={n} seed={seed} unsorted: {a:?}",
            );
            for (i, &v) in a.iter().enumerate() {
                assert_eq!(v, i as u64, "QHS+SQS: n={n} seed={seed} missing");
            }
        }
    }
}

/// Cycle-bounded concrete type: `HeapExtract` whose heap build itself runs
/// `SequentialSet<RecursiveQuickSelect<HeapExtract<...>, NoPivot>>` — the
/// SetQuickSelect-driven cycle replacement for the old `PartitionDrivenDeepHeapify`.
/// The head-count rule forces the *inner* HeapExtract's DH slot to a leaf
/// (Iterative here). This test confirms the cycle-bounded type
/// instantiates AND sorts correctly — slow (O(n²) territory), so we cap
/// the sizes.
#[test]
fn cycle_bounded_set_quick_select_heapextract_sorts() {
    type InnerHE = HeapExtract<AryPair<Binary>, Iterative>;
    type Cycle =
        HeapExtract<AryPair<Binary>, SequentialSet<RecursiveQuickSelect<InnerHE, NoPivot>>>;
    type Sort = QuickSort<Cycle, NoPivot, NoSmallSort>;

    for &n in &[0usize, 1, 2, 3, 7, 16, 50, 128] {
        for seed in 0..2u64 {
            let mut a = shuffled(n, seed);
            Sort::sort(&mut a, &mut NoOpLogger);
            assert!(
                a.windows(2).all(|w| w[0] <= w[1]),
                "cycle-bounded sort: n={n} seed={seed} unsorted: {a:?}",
            );
            for (i, &v) in a.iter().enumerate() {
                assert_eq!(v, i as u64, "cycle-bounded sort: n={n} seed={seed} missing");
            }
        }
    }
}
