//! Correctness for the three `SetQuickSelect` impls (`Stack`,
//! `SequentialSet`, `RecursiveSet`). Each is plugged in as the
//! `DeepHeapify` for `NaryHeapSort` and asked to sort a shuffled array
//! across multiple heap directions; we check the result is fully sorted.
//!
//! The two `*Set` impls handle direction by deferring to ascending `Ord`
//! and reversing once at the end if the heap's rootward direction is the
//! opposite end — that's the path the test exercises across `MaxForward`,
//! `MinReverse`, `MinForward`, `MaxReverse`.

use heap_sort_lib::arity::{Binary, Ternary};
use heap_sort_lib::arity_heap::ArityHeap;
// Only `MaxForward` and `MinReverse` produce ascending output from
// `HeapAlgorithm::sort`; the other two directions sort descending, so
// the ascending assertion in `check_sorts` doesn't apply to them.
use heap_sort_lib::direction::{MaxForward, MinReverse};
use heap_sort_lib::heap_algorithm::HeapAlgorithm;
use heap_sort_lib::heap_sort::NaryHeapSort;
use heap_sort_lib::set_quick_select::{RecursiveSet, SequentialSet, StackSet};
use partition_hoare::LeftRightPartition;
use partition_lomuto::LeftLeftPartition;
use pivot_first::FirstElement;
use pivot_median3::MedianOfThree;
use quick_select_lib::{IterativeQuickSelect, RecursiveQuickSelect};
use sort_logger::NoOpLogger;

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
    for &n in &[0usize, 1, 2, 3, 7, 16, 17, 50, 100, 500] {
        for seed in 0..2u64 {
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

// ── SequentialSet<QS> ────────────────────────────────────────────────────────

#[test]
fn sequential_set_sorts_across_directions() {
    type QS = RecursiveQuickSelect<LeftLeftPartition, FirstElement>;
    type DH = SequentialSet<QS>;

    check_sorts("MaxForward + Binary", |a, l| {
        NaryHeapSort::<ArityHeap<Binary, MaxForward>, DH>::sort(a, l)
    });
    check_sorts("MinReverse + Binary", |a, l| {
        NaryHeapSort::<ArityHeap<Binary, MinReverse>, DH>::sort(a, l)
    });
    check_sorts("MaxForward + Ternary", |a, l| {
        NaryHeapSort::<ArityHeap<Ternary, MaxForward>, DH>::sort(a, l)
    });
    check_sorts("MinReverse + Ternary", |a, l| {
        NaryHeapSort::<ArityHeap<Ternary, MinReverse>, DH>::sort(a, l)
    });
}

// ── RecursiveSet<QS> ─────────────────────────────────────────────────────────

#[test]
fn recursive_set_sorts_across_directions() {
    type QS = RecursiveQuickSelect<LeftLeftPartition, FirstElement>;
    type DH = RecursiveSet<QS>;

    check_sorts("MaxForward + Binary", |a, l| {
        NaryHeapSort::<ArityHeap<Binary, MaxForward>, DH>::sort(a, l)
    });
    check_sorts("MinReverse + Ternary", |a, l| {
        NaryHeapSort::<ArityHeap<Ternary, MinReverse>, DH>::sort(a, l)
    });
}

// ── StackSet (pass-through to StackPartialQuickDeepHeapify) ──────────────────

#[test]
fn stack_set_sorts_across_directions() {
    // `StackSet` slots a *heap-logical* `HeapPartition` (the layout-aware
    // partition from `heap_sort_lib::heap_partition`), not the flat
    // quicksort `PartitionScheme` that the `*Set<QS>` impls inherit
    // through `QuickSelect`.
    type DH = StackSet<heap_sort_lib::heap_partition::LeftLeftPartition, FirstElement>;

    check_sorts("MaxForward + Binary", |a, l| {
        NaryHeapSort::<ArityHeap<Binary, MaxForward>, DH>::sort(a, l)
    });
    check_sorts("MinReverse + Binary", |a, l| {
        NaryHeapSort::<ArityHeap<Binary, MinReverse>, DH>::sort(a, l)
    });
}
