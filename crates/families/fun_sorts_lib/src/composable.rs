//! Compositional complexity annotations for the fun-sorts types.
//!
//! The spec emit reads `<Ty as HasTimeBounds>::WORST` / `HasSpace::SPACE` /
//! `HasStability::STABLE` from every emittable type. The legacy `sort_family!` /
//! TOML registrations carried a single `big_o` STRING instead; these impls pin
//! the same classes as trait consts so the spec catalog can emit fun sorts.
//!
//! These are pedagogical / adversarial sorts, so the bounds are FIXED per type
//! (not composed from the partition/pivot axis) — `WORST = BEST = AVERAGE` at the
//! legacy `big_o` class, matching what `Complexity::from_str` produced. SPACE is
//! a conservative upper bound (recursion stack `LOG_N`; an aux buffer / partition
//! scratch `N`); STABLE is `false` throughout (none preserves equal-key order).

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::{
    Complexity, NonTrivialSmallSort, PartitionScheme, QuickSelect, SmallSort, Special,
};
use gap_distribution_lib::GapDistribution;

use crate::{
    BadHeapSort, BadHeapSortAlt, CyclentSort, CyclentSortOpt, CyclentSortStack,
    CyclentSortStackOptimized, QuickSurrender, QuickSurrenderOptimised, RandomShellSort, SlowSort,
    SlowSortPotzko, StoogeSort,
};

/// `O(N^2.71)` stooge-sort class — bucketed to `N² · √N` (the closest class the
/// `Complexity` struct represents), matching `Complexity::from_str("O(N^2.71)")`.
const N_POW_2_71: Complexity = Complexity { n_pow: 2, log_pow: 0, special: Some(Special::Sqrt) };

/// Impl the three composable traits for a non-generic type at a fixed class.
macro_rules! fixed {
    ($ty:ty, $worst:expr, $space:expr) => {
        impl HasTimeBounds for $ty {
            const WORST: Complexity = $worst;
            const BEST: Complexity = $worst;
            const AVERAGE: Complexity = $worst;
        }
        impl HasSpace for $ty {
            const SPACE: Complexity = $space;
        }
        impl HasStability for $ty {
            const STABLE: bool = false;
        }
    };
}

/// Same, for a one-parameter generic type bounded on `$bound`.
macro_rules! fixed_generic1 {
    ($ty:ident, $bound:path, $worst:expr, $space:expr) => {
        impl<X: $bound> HasTimeBounds for $ty<X> {
            const WORST: Complexity = $worst;
            const BEST: Complexity = $worst;
            const AVERAGE: Complexity = $worst;
        }
        impl<X: $bound> HasSpace for $ty<X> {
            const SPACE: Complexity = $space;
        }
        impl<X: $bound> HasStability for $ty<X> {
            const STABLE: bool = false;
        }
    };
}

// ── zero-axis sorts ──────────────────────────────────────────────────────────
fixed!(SlowSort, Complexity::EXPONENTIAL, Complexity::LOG_N); // O(N^logN) → exp
fixed!(SlowSortPotzko, Complexity::EXPONENTIAL, Complexity::LOG_N); // T(n)=2T(n-1)
fixed!(BadHeapSort, Complexity::N_SQUARED, Complexity::LOG_N); // ~N² (unanalysed)
fixed!(BadHeapSortAlt, Complexity::N_SQUARED, Complexity::LOG_N);

// ── cyclent family (generic over the inner PartitionScheme) ──────────────────
fixed_generic1!(CyclentSort, PartitionScheme, Complexity::CUBIC, Complexity::N1);
fixed_generic1!(CyclentSortOpt, PartitionScheme, Complexity::N_SQUARED, Complexity::N1);
fixed_generic1!(CyclentSortStack, PartitionScheme, Complexity::N_LOG_N, Complexity::N1);
fixed_generic1!(
    CyclentSortStackOptimized,
    PartitionScheme,
    Complexity::N_LOG_N,
    Complexity::N1
);

// ── stooge (generic over SmallSort) ──────────────────────────────────────────
fixed_generic1!(StoogeSort, SmallSort, N_POW_2_71, Complexity::LOG_N);

// ── random shell (generic over GapDistribution; allocates a gap array) ───────
fixed_generic1!(RandomShellSort, GapDistribution, Complexity::N_SQRT_N, Complexity::N1);

// ── quick surrender (generic over the inner QuickSelect; +small for optimised)
fixed_generic1!(QuickSurrender, QuickSelect, Complexity::N_SQUARED, Complexity::N1);

impl<QS: QuickSelect, SS: NonTrivialSmallSort> HasTimeBounds for QuickSurrenderOptimised<QS, SS> {
    const WORST: Complexity = Complexity::N_LOG_N;
    const BEST: Complexity = Complexity::N_LOG_N;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
}
impl<QS: QuickSelect, SS: NonTrivialSmallSort> HasSpace for QuickSurrenderOptimised<QS, SS> {
    const SPACE: Complexity = Complexity::N1;
}
impl<QS: QuickSelect, SS: NonTrivialSmallSort> HasStability for QuickSurrenderOptimised<QS, SS> {
    const STABLE: bool = false;
}
