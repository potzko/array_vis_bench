//! `CorrectnessSuite`, a correctness battery as a per kind construct.
//!
//! The spec compiler emits, for every algorithm leaf, a `run_correctness: fn()`
//! pointer that the [`crate::mains::Correctness`] consumer calls uniformly. This
//! module answers which battery that pointer runs: a trait, one impl per kind,
//! generic over the kind's role trait.
//!
//! ```text
//! impl<Q: QuickSelect> CorrectnessSuite for SelectSuite<Q> { ... quick_select_battery ... }
//! //   ^ the role trait        ^ the kind's suite            ^ the kind's battery
//! ```
//!
//! Adding a first class kind is a `<Kind>Suite` impl here, a row of `spec_emit`'s
//! `KINDS` table and a `Category` variant. Every emitted `run_correct` is
//! `<{Kind}Suite<Ty> as CorrectnessSuite>::verify(NAME, cap)`, so the
//! `Correctness` main stays kind agnostic.

use std::marker::PhantomData;

use array_vis_bench_traits::{
    DirectSort, HasStability, PartitionScheme, QuickSelect, Rotation, RotationMerge, ScratchspaceRun,
    SmallSort,
};
use sort_logger::NoOpLogger;

use crate::bench_registry::correctness;

/// The correctness battery for one algorithm KIND. One impl per kind, each
/// parameterised over its generic algorithm class; the spec emit picks the
/// impl by the leaf's catalog `category` and calls [`CorrectnessSuite::verify`]
/// from the leaf's `run_correctness` pointer.
pub trait CorrectnessSuite {
    /// Run this kind's battery for the entry named `name`, whose
    /// `max_input_size` is `cap`. Panics on a verification failure (the
    /// `Correctness` main catches the panic).
    fn verify(name: &str, cap: Option<usize>);
}

/// Sort suite: drives [`correctness::sort_battery`] and
/// [`correctness::sort_stability_battery`] through the concrete `S: DirectSort`,
/// reading its stability off the type.
pub struct SortSuite<S>(PhantomData<S>);

impl<S: DirectSort + HasStability> CorrectnessSuite for SortSuite<S> {
    fn verify(name: &str, cap: Option<usize>) {
        fn run<S: DirectSort>(arr: &mut [usize], _index: usize, logger: &mut NoOpLogger) -> Option<(usize, usize)> {
            S::sort(arr, logger);
            None
        }
        correctness::sort_battery(run::<S>, name, cap);
        correctness::sort_stability_battery(run::<S>, name, S::STABLE, cap);
    }
}

/// Quick-select suite: drives [`correctness::quick_select_battery`] through the
/// concrete `Q: QuickSelect`'s role-trait `select`, with the same empty-guard +
/// target-clamp the standalone registration uses.
pub struct SelectSuite<Q>(PhantomData<Q>);

impl<Q: QuickSelect> CorrectnessSuite for SelectSuite<Q> {
    fn verify(name: &str, cap: Option<usize>) {
        fn noop<Q: QuickSelect>(arr: &mut [usize], target: usize, logger: &mut NoOpLogger) -> Option<(usize, usize)> {
            if arr.is_empty() {
                return None;
            }
            let t = target.min(arr.len() - 1);
            <Q as QuickSelect>::select(arr, logger, t);
            None
        }
        correctness::quick_select_battery(noop::<Q>, name, cap);
    }
}

/// Rotation suite: drives [`correctness::rotation_battery`] through the
/// concrete `R: Rotation`'s role trait via the shared `run_rotation`
/// one-shot (which allocates/frees the rotation's scratch buffer).
pub struct RotationSuite<R>(PhantomData<R>);

impl<R: Rotation> CorrectnessSuite for RotationSuite<R> {
    fn verify(name: &str, cap: Option<usize>) {
        fn run<R: Rotation>(arr: &mut [usize], split: usize, logger: &mut NoOpLogger) -> Option<(usize, usize)> {
            array_vis_bench_traits::role::rotation::run_rotation::<R, usize, NoOpLogger>(
                arr, split, logger,
            );
            None
        }
        correctness::rotation_battery(run::<R>, name, cap);
    }
}

/// Small-sort suite: drives [`correctness::small_sort_battery`] through the
/// concrete `S: SmallSort`'s role-trait `sort`. The entry's cap is the leaf's
/// threshold, so the battery only feeds inputs the contract covers.
pub struct SmallSortSuite<S>(PhantomData<S>);

impl<S: SmallSort> CorrectnessSuite for SmallSortSuite<S> {
    fn verify(name: &str, cap: Option<usize>) {
        fn run<S: SmallSort>(arr: &mut [usize], _index: usize, logger: &mut NoOpLogger) -> Option<(usize, usize)> {
            let _ = <S as SmallSort>::sort(arr, logger);
            None
        }
        correctness::small_sort_battery(run::<S>, name, cap);
    }
}

/// Merge suite: drives [`correctness::merge_battery`] through the concrete
/// `M: RotationMerge` via the shared `run_rotation_merge` one-shot (which
/// allocates/frees the merge's scratch buffer).
pub struct MergeSuite<M>(PhantomData<M>);

impl<M: RotationMerge> CorrectnessSuite for MergeSuite<M> {
    fn verify(name: &str, cap: Option<usize>) {
        fn run<M: RotationMerge>(arr: &mut [usize], mid: usize, logger: &mut NoOpLogger) -> Option<(usize, usize)> {
            array_vis_bench_traits::role::merge::run_rotation_merge::<M, usize, NoOpLogger>(
                arr, mid, logger,
            );
            None
        }
        correctness::merge_battery(run::<M>, name, cap);
    }
}

/// Partition suite: drives [`correctness::partition_battery`] through a
/// self-pivoting `P: PartitionScheme` (typically `Pivoted<Scheme, Sel>`) via the
/// shared `run_partition_bounds`, which returns the `(left_end, right_start)`
/// boundary the battery checks.
pub struct PartitionSuite<P>(PhantomData<P>);

impl<P: PartitionScheme> CorrectnessSuite for PartitionSuite<P> {
    fn verify(name: &str, cap: Option<usize>) {
        fn run<P: PartitionScheme>(arr: &mut [usize], _index: usize, logger: &mut NoOpLogger) -> Option<(usize, usize)> {
            Some(array_vis_bench_traits::role::partition::run_partition_bounds::<P, usize, NoOpLogger>(
                arr, logger,
            ))
        }
        correctness::partition_battery(run::<P>, name, cap);
    }
}

/// Scratchspace suite: drives [`correctness::scratchspace_battery`] through the
/// concrete workflow `K: ScratchspaceRun` (which allocates its own auxiliary and
/// runs a scripted routine over the ops). Every workflow nets to identity, so the
/// battery verifies the whole contract round-trips faithfully.
pub struct ScratchspaceSuite<K>(PhantomData<K>);

impl<K: ScratchspaceRun> CorrectnessSuite for ScratchspaceSuite<K> {
    fn verify(name: &str, cap: Option<usize>) {
        fn run<K: ScratchspaceRun>(arr: &mut [usize], _index: usize, logger: &mut NoOpLogger) -> Option<(usize, usize)> {
            K::run(arr, logger);
            None
        }
        correctness::scratchspace_battery(run::<K>, name, cap);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use array_vis_bench_traits::{NoPivot, QuickSelect};
    use sort_logger::SortLogger;

    /// A trivial in-module QuickSelect impl (selection by full sort) so the
    /// suite can be exercised without pulling a family crate into core's
    /// dev-deps. Proves `SelectSuite<Q>` wires the battery to `Q::select`.
    struct SortingSelect;
    impl QuickSelect for SortingSelect {
        fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
            arr: &mut [T],
            _logger: &mut U,
            _target: usize,
        ) {
            arr.sort();
        }
    }

    #[test]
    fn select_suite_runs_the_quick_select_battery_for_a_valid_impl() {
        // A correct selector passes the battery (no panic).
        SelectSuite::<SortingSelect>::verify("test: sorting select", None);
    }

    // A sort by `slice::sort`, and one that does nothing, so the sort suite can be
    // exercised without pulling a family crate into core's dev-deps.
    struct StdSort;
    impl DirectSort for StdSort {
        fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], _logger: &mut U) {
            arr.sort();
        }
    }
    impl HasStability for StdSort {
        const STABLE: bool = true;
    }
    struct IdleSort;
    impl DirectSort for IdleSort {
        fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_arr: &mut [T], _logger: &mut U) {}
    }
    impl HasStability for IdleSort {
        const STABLE: bool = false;
    }
    // Panics on an array past eight, so a battery that ignores its cap fails.
    struct EightSort;
    impl DirectSort for EightSort {
        fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], _logger: &mut U) {
            assert!(arr.len() <= 8, "fed {} elements past the cap", arr.len());
            arr.sort();
        }
    }
    impl HasStability for EightSort {
        const STABLE: bool = true;
    }

    #[test]
    fn sort_suite_runs_the_sort_batteries_for_a_valid_impl() {
        SortSuite::<StdSort>::verify("test: std sort", None);
    }

    #[test]
    #[should_panic(expected = "failed on")]
    fn sort_suite_fails_a_sort_that_does_nothing() {
        SortSuite::<IdleSort>::verify("test: idle sort", None);
    }

    #[test]
    fn sort_suite_hands_its_cap_to_the_batteries() {
        SortSuite::<EightSort>::verify("test: eight sort", Some(8));
    }

    // A merge by full sort, and one that does nothing, so the suite can be
    // exercised without pulling a family crate into core's dev-deps.
    struct SortingMerge;
    impl RotationMerge for SortingMerge {
        fn scratch_size(_n: usize) -> usize {
            0
        }
        fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
            arr: &mut [T],
            _mid: usize,
            _scratch: &mut [T],
            _logger: &mut U,
        ) {
            arr.sort();
        }
    }
    struct IdleMerge;
    impl RotationMerge for IdleMerge {
        fn scratch_size(_n: usize) -> usize {
            0
        }
        fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
            _arr: &mut [T],
            _mid: usize,
            _scratch: &mut [T],
            _logger: &mut U,
        ) {
        }
    }

    #[test]
    fn merge_suite_runs_the_merge_battery_for_a_valid_impl() {
        MergeSuite::<SortingMerge>::verify("test: sorting merge", None);
    }

    #[test]
    #[should_panic(expected = "merge failed")]
    fn merge_suite_fails_a_merge_that_leaves_the_runs_apart() {
        MergeSuite::<IdleMerge>::verify("test: idle merge", None);
    }

    #[test]
    fn no_pivot_is_in_scope_for_heap_extract_selectors() {
        // Smoke: the role types the heap-extract quick-selects compose with
        // are importable from the traits crate.
        let _ = PhantomData::<NoPivot>;
    }
}
