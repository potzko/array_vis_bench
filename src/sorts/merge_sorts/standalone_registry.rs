//! Register rotation-based merges as standalone `Category::Merge`
//! algorithms — visualisable, testable, benchable through the same
//! pipeline as sorts. Pure cross-product registration; the merge
//! algorithms themselves live in `rotation_merge.rs`.
//!
//! Two merge families × eleven rotations = twenty-two leaves. Each
//! leaf gets its own private inner `mod` so duplicate identifiers
//! (`ENTRY`, `merge_dyn`, etc.) don't collide across invocations.

use crate::sorts::merge_sorts::rotation_merge::{NaiveRotationMerge, SmallerSideRotationMerge};
use crate::utils::rotation::{
    AuxiliaryRotation, BridgeRotation, ContrevRotation, DrillRotation, GrailRotation,
    GriesMillsRotation, HelixRotation, JugglingRotation, PistonRotation, ReversalRotation,
    TrinityRotation,
};

/// One leaf: a `RotationMerge` impl wrapped so it appears as a
/// standalone algorithm under `/merges/<family>/<rotation>/` in the
/// menu tree.
macro_rules! register_merge {
    ($mod:ident, $merge:ty, $family:expr, $rot:ty) => {
        mod $mod {
            use super::*;
            use crate::traits::log_traits::{NoOpLogger, SortLogger};

            const ROT_NAME: &str = <$rot as crate::utils::rotation::Rotation>::NAME;
            const NAME: &str = const_format::concatcp!(
                "merge: ", $family, "<", ROT_NAME, ">",
            );

            fn run_once<U: ?Sized + SortLogger<usize>>(
                arr: &mut [usize],
                mid: usize,
                logger: &mut U,
            ) {
                use crate::sorts::merge_sorts::rotation_merge::RotationMerge;
                let scratch_size = <$merge>::scratch_size(arr.len());
                if scratch_size == 0 {
                    <$merge>::merge(arr, mid, &mut [], logger);
                } else {
                    let mut scratch = logger.create_aux_arr_t(scratch_size);
                    <$merge>::merge(arr, mid, &mut scratch, logger);
                    logger.free_aux_arr_t(&scratch);
                }
            }

            fn merge_dyn(
                arr: &mut [usize],
                mid: usize,
                logger: &mut dyn SortLogger<usize>,
            ) {
                run_once(arr, mid, logger)
            }
            fn merge_noop(
                arr: &mut [usize],
                mid: usize,
                logger: &mut NoOpLogger,
            ) {
                run_once(arr, mid, logger)
            }

            fn run_with_input(
                input_name: &str,
                config: &crate::bench_registry::RunConfig,
                logger: &mut dyn SortLogger<usize>,
            ) {
                crate::bench_registry::run_merge_with_input(
                    input_name, config, merge_dyn, logger,
                );
            }

            fn run_correctness() {
                crate::bench_registry::correctness::merge_battery(merge_noop, NAME);
            }

            #[linkme::distributed_slice(crate::bench_registry::ALGORITHMS)]
            pub(super) static ENTRY: crate::bench_registry::AlgorithmEntry =
                crate::bench_registry::AlgorithmEntry {
                    name: NAME,
                    category: crate::bench_registry::Category::Merge,
                    big_o: "O(N log N)",
                    stable: false,
                    max_input_size: None,
                    run_with_input,
                    run_correctness,
                };

            #[ctor::ctor]
            fn register_path() {
                sort_registry_core::register_sort_path(
                    NAME,
                    "O(N log N)",
                    false,
                    &["merges", $family, ROT_NAME],
                );
            }

            #[cfg(test)]
            mod merge_test {
                #[test]
                fn correctness() {
                    crate::bench_registry::test_helpers::check_sort_subprocess_assert(
                        &super::ENTRY,
                        crate::bench_registry::test_helpers::DEFAULT_TIMEOUT,
                    );
                }
            }
        }
    };
}

/// Expand once per rotation for a given merge family.
macro_rules! register_naive_merges {
    ($($mod:ident => $rot:ident),* $(,)?) => {
        $(
            register_merge!(
                $mod,
                NaiveRotationMerge<$rot>,
                "naive",
                $rot
            );
        )*
    };
}

macro_rules! register_smaller_side_merges {
    ($($mod:ident => $rot:ident),* $(,)?) => {
        $(
            register_merge!(
                $mod,
                SmallerSideRotationMerge<$rot>,
                "smaller-side",
                $rot
            );
        )*
    };
}

register_naive_merges! {
    naive_reversal      => ReversalRotation,
    naive_auxiliary     => AuxiliaryRotation,
    naive_bridge        => BridgeRotation,
    naive_contrev       => ContrevRotation,
    naive_trinity       => TrinityRotation,
    naive_gries_mills   => GriesMillsRotation,
    naive_grail         => GrailRotation,
    naive_piston        => PistonRotation,
    naive_helix         => HelixRotation,
    naive_drill         => DrillRotation,
    naive_juggling      => JugglingRotation,
}

register_smaller_side_merges! {
    ss_reversal      => ReversalRotation,
    ss_auxiliary     => AuxiliaryRotation,
    ss_bridge        => BridgeRotation,
    ss_contrev       => ContrevRotation,
    ss_trinity       => TrinityRotation,
    ss_gries_mills   => GriesMillsRotation,
    ss_grail         => GrailRotation,
    ss_piston        => PistonRotation,
    ss_helix         => HelixRotation,
    ss_drill         => DrillRotation,
    ss_juggling      => JugglingRotation,
}

// ── Auxiliary-array merges ───────────────────────────────────────────────────
//
// Parallel to the rotation merges above: an `AuxMerge` trait (in
// `auxiliary_merge.rs`) with concrete impls. Each impl is registered
// under `/merges/auxiliary/<variant>/`.

use crate::sorts::merge_sorts::auxiliary_merge::{AuxMerge, FullCopyAuxMerge, HalfCopyAuxMerge};

macro_rules! register_aux_merge {
    ($mod:ident, $merge:ty) => {
        mod $mod {
            use super::*;
            use crate::traits::log_traits::{NoOpLogger, SortLogger};

            const VARIANT: &str = <$merge as AuxMerge>::NAME;
            const NAME: &str = const_format::concatcp!(
                "merge: auxiliary<", VARIANT, ">",
            );

            fn merge_dyn(
                arr: &mut [usize],
                mid: usize,
                logger: &mut dyn SortLogger<usize>,
            ) {
                <$merge as AuxMerge>::merge(arr, mid, logger)
            }
            fn merge_noop(
                arr: &mut [usize],
                mid: usize,
                logger: &mut NoOpLogger,
            ) {
                <$merge as AuxMerge>::merge(arr, mid, logger)
            }

            fn run_with_input(
                input_name: &str,
                config: &crate::bench_registry::RunConfig,
                logger: &mut dyn SortLogger<usize>,
            ) {
                crate::bench_registry::run_merge_with_input(
                    input_name, config, merge_dyn, logger,
                );
            }

            fn run_correctness() {
                crate::bench_registry::correctness::merge_battery(merge_noop, NAME);
            }

            #[linkme::distributed_slice(crate::bench_registry::ALGORITHMS)]
            pub(super) static ENTRY: crate::bench_registry::AlgorithmEntry =
                crate::bench_registry::AlgorithmEntry {
                    name: NAME,
                    category: crate::bench_registry::Category::Merge,
                    big_o: "O(N)",
                    stable: true,
                    max_input_size: None,
                    run_with_input,
                    run_correctness,
                };

            #[ctor::ctor]
            fn register_path() {
                sort_registry_core::register_sort_path(
                    NAME,
                    "O(N)",
                    true,
                    &["merges", "auxiliary", VARIANT],
                );
            }

            #[cfg(test)]
            mod merge_test {
                #[test]
                fn correctness() {
                    crate::bench_registry::test_helpers::check_sort_subprocess_assert(
                        &super::ENTRY,
                        crate::bench_registry::test_helpers::DEFAULT_TIMEOUT,
                    );
                }
            }
        }
    };
}

register_aux_merge!(aux_full, FullCopyAuxMerge);
register_aux_merge!(aux_half, HalfCopyAuxMerge);
