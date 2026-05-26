//! Property-based tests run alongside each algorithm's fixed correctness
//! battery, plus an aggregate determinism check.
//!
//! Compiled only under `#[cfg(test)]` so `proptest` stays a dev-dependency.
//!
//! ## Layout
//!
//! Each per-property module owns one property and exposes a single
//! `run(fn_ptr, name, ...)` entry point. The matching
//! `correctness::*_battery` calls it after the fixed bank, so every
//! algorithm gets both curated and generated coverage per
//! `cargo test` run.
//!
//! The aggregate trace-determinism check lives in `determinism.rs` as a
//! top-level `#[test]`.

pub mod sort;
pub mod rotation;
pub mod partition;
pub mod merge;
pub mod quick_select;
pub mod small_sort;
mod determinism;

use linkme::distributed_slice;
use proptest::prelude::*;
use proptest::test_runner::{Config, TestRunner};

use crate::bench_registry::{
    max_n_for_tests, MergePropHook, PartitionPropHook, QuickSelectPropHook, RotationPropHook,
    SmallSortPropHook, SortPropHook, MERGE_PROP_HOOKS, PARTITION_PROP_HOOKS,
    QUICK_SELECT_PROP_HOOKS, ROTATION_PROP_HOOKS, SMALL_SORT_PROP_HOOKS, SORT_PROP_HOOKS,
};

// ── Property-test hooks (test-only) ─────────────────────────────────────────
//
// The category batteries in `array_vis_bench_core::bench_registry` iterate
// these slices after the fixed bank. With this module only compiled under
// cfg(test), each category's `run` fn registers as one hook so proptest
// runs as part of every `cargo test` correctness invocation, and zero
// proptest references leak into release builds.

#[distributed_slice(SORT_PROP_HOOKS)]
static SORT_HOOK: SortPropHook = sort::run;
#[distributed_slice(ROTATION_PROP_HOOKS)]
static ROTATION_HOOK: RotationPropHook = rotation::run;
#[distributed_slice(PARTITION_PROP_HOOKS)]
static PARTITION_HOOK: PartitionPropHook = partition::run;
#[distributed_slice(MERGE_PROP_HOOKS)]
static MERGE_HOOK: MergePropHook = merge::run;
#[distributed_slice(QUICK_SELECT_PROP_HOOKS)]
static QUICK_SELECT_HOOK: QuickSelectPropHook = quick_select::run;
#[distributed_slice(SMALL_SORT_PROP_HOOKS)]
static SMALL_SORT_HOOK: SmallSortPropHook = small_sort::run;

/// Per-(algorithm × property) random-case budget.
pub(crate) const CASES: u32 = 64;

/// Default cap on generated N when no `register_test_cap!` applies.
pub(crate) const DEFAULT_MAX_N: usize = 300;

/// Lower bound for the proptest size strategy. Below 2, cases degenerate
/// to trivial inputs the fixed bank already covers.
pub(crate) const MIN_N: usize = 2;

/// Per-algorithm N ceiling, clamped to `DEFAULT_MAX_N`. Generated cases
/// at this scale are about diversity, not stress, so never exceed the
/// default even if the algorithm declares a larger cap.
pub(crate) fn n_cap(name: &str) -> usize {
    max_n_for_tests(name).unwrap_or(DEFAULT_MAX_N).min(DEFAULT_MAX_N)
}

pub(crate) fn runner() -> TestRunner {
    TestRunner::new(Config {
        cases: CASES,
        // Shrink budget kept modest — sort regressions usually minimise
        // quickly; we want fast failure reporting, not exhaustive shrinking.
        max_shrink_iters: 256,
        failure_persistence: None,
        ..Config::default()
    })
}

/// Run `body` against `strategy`; panic with the property name on failure.
/// Centralises the runner / error boilerplate so each property file only
/// describes its strategy and assertions.
pub(crate) fn check<S, F>(name: &str, property: &str, strategy: S, body: F)
where
    S: Strategy,
    F: Fn(S::Value) -> Result<(), TestCaseError>,
{
    if let Err(e) = runner().run(&strategy, body) {
        panic!("{name}: {property} proptest failed: {e}");
    }
}

/// `Vec<usize>` of length `MIN_N..=max_n` with enough duplicates
/// (values in `0..(n*4).max(8)`) to exercise equal-key paths and
/// stable-sort invariants.
pub(crate) fn vec_strategy(max_n: usize) -> impl Strategy<Value = Vec<usize>> {
    (MIN_N..=max_n).prop_flat_map(|n| {
        let max_val = (n * 4).max(8);
        prop::collection::vec(0usize..max_val, n)
    })
}
