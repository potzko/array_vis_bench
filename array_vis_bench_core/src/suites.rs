//! `CorrectnessSuite` — a correctness battery as a per-KIND construct.
//!
//! The spec compiler emits, for every algorithm leaf, a `run_correctness: fn()`
//! pointer that the [`crate::mains::Correctness`] consumer calls uniformly. The
//! question this module answers is *which* battery that pointer runs — and it
//! answers it the way the workspace models everything else: as a **trait, one
//! impl per kind**, parameterised over the kind's generic algorithm class.
//!
//! ```text
//! impl<Q: QuickSelect> CorrectnessSuite for SelectSuite<Q> { … quick_select_battery … }
//! //   ^ the role trait        ^ the kind's suite            ^ the kind's battery
//! ```
//!
//! Adding a first-class kind (rotation, merge, partition, …) is then just
//! another `impl CorrectnessSuite for XSuite<R>` plus a thin emit arm — never a
//! new hardcoded battery `match`. The `Correctness` main stays kind-agnostic: it
//! only ever calls the entry's fn pointer, which the emit wires to the right
//! suite.
//!
//! ASYMMETRY (deliberate, documented): full **sorts** have no `sort` role trait
//! — the modern direct-sort shape exposes only an *inherent*
//! `fn sort<T, U: ?Sized + SortLogger<T>>` (the workspace avoids the legacy
//! `SortAlgo`, whose `U: Sized` can't take a `dyn` logger). A trait impl can't
//! reach an inherent method generically, so the sort emit keeps calling
//! `correctness::sort_battery` directly via the monomorphic adapter it already
//! generates. Quick-select (and the other role-trait-backed kinds) *do* have a
//! role trait, so they go through `CorrectnessSuite`. Unifying sorts too would
//! mean introducing a `DirectSort` role trait every sort type implements — a
//! larger, separate change.

use std::marker::PhantomData;

use array_vis_bench_traits::QuickSelect;
use sort_logger::NoOpLogger;

use crate::bench_registry::correctness;

/// The correctness battery for one algorithm KIND. One impl per kind, each
/// parameterised over its generic algorithm class; the spec emit picks the
/// impl by the leaf's catalog `category` and calls [`CorrectnessSuite::verify`]
/// from the leaf's `run_correctness` pointer.
pub trait CorrectnessSuite {
    /// Run this kind's battery for the entry named `name`. Panics on a
    /// verification failure (the `Correctness` main catches the panic).
    fn verify(name: &str);
}

/// Quick-select suite: drives [`correctness::quick_select_battery`] through the
/// concrete `Q: QuickSelect`'s role-trait `select`, with the same empty-guard +
/// target-clamp the standalone registration uses.
pub struct SelectSuite<Q>(PhantomData<Q>);

impl<Q: QuickSelect> CorrectnessSuite for SelectSuite<Q> {
    fn verify(name: &str) {
        fn noop<Q: QuickSelect>(arr: &mut [usize], target: usize, logger: &mut NoOpLogger) {
            if arr.is_empty() {
                return;
            }
            let t = target.min(arr.len() - 1);
            <Q as QuickSelect>::select(arr, logger, t);
        }
        correctness::quick_select_battery(noop::<Q>, name);
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
        SelectSuite::<SortingSelect>::verify("test: sorting select");
    }

    #[test]
    fn no_pivot_is_in_scope_for_heap_extract_selectors() {
        // Smoke: the role types the heap-extract quick-selects compose with
        // are importable from the traits crate.
        let _ = PhantomData::<NoPivot>;
    }
}
