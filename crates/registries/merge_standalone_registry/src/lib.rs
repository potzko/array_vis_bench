//! Register rotation-based merges as standalone `Category::Merge`
//! algorithms — visualisable, testable, benchable through the same
//! pipeline as sorts. Pure cross-product registration; the merge
//! algorithms themselves live in `merge_sort_lib::rotation_merge`.
//!
//! Two merge families × eleven rotations = twenty-two leaves. Each
//! leaf gets its own private inner `mod` so duplicate identifiers
//! (`ENTRY`, `merge_dyn`, etc.) don't collide across invocations.
//!
//! This crate has no public API beyond [`LINK_ANCHOR`] — its job is the
//! `#[ctor]` + `#[linkme::distributed_slice]` side-effects that fire
//! when it's linked. Downstream wiring crates reference [`LINK_ANCHOR`]
//! from a `#[used]` static so the linker doesn't drop the object file
//! under `--gc-sections`.

/// Force-link anchor — see module docs.
pub static LINK_ANCHOR: () = ();

use merge_sort_lib::rotation_merge::{NaiveRotationMerge, SmallerSideRotationMerge};
use rotation_auxiliary::AuxiliaryRotation;
use rotation_bridge::BridgeRotation;
use rotation_contrev::ContrevRotation;
use rotation_drill::DrillRotation;
use rotation_grail::GrailRotation;
use rotation_gries_mills::GriesMillsRotation;
use rotation_helix::HelixRotation;
use rotation_juggling::JugglingRotation;
use rotation_piston::PistonRotation;
use rotation_reversal::ReversalRotation;
use rotation_trinity::TrinityRotation;

/// One leaf: a `RotationMerge` impl wrapped so it appears as a
/// standalone algorithm under `/merges/<family>/<rotation>/` in the
/// menu tree.
macro_rules! register_merge {
    ($mod:ident, $merge:ty, $family:expr, $rot:ty) => {
        mod $mod {
            use super::*;
            use sort_logger::{NoOpLogger, SortLogger};

            const ROT_NAME: &str = <$rot as array_vis_bench_traits::Rotation>::NAME;
            const NAME: &str = const_format::concatcp!(
                "merge: ", $family, "<", ROT_NAME, ">",
            );

            fn run_once<U: ?Sized + SortLogger<usize>>(
                arr: &mut [usize],
                mid: usize,
                logger: &mut U,
            ) {
                use merge_sort_lib::rotation_merge::RotationMerge;
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
                config: &array_vis_bench_core::bench_registry::RunConfig,
                logger: &mut dyn SortLogger<usize>,
            ) {
                array_vis_bench_core::bench_registry::run_merge_with_input(
                    input_name, config, merge_dyn, logger,
                );
            }

            fn run_correctness() {
                array_vis_bench_core::bench_registry::correctness::merge_battery(merge_noop, NAME);
            }

            // Pull every structured field straight from the merge type's
            // compositional impls — Naive contributes O(N²) worst, Smaller
            // Side contributes O(N log² N), and both forward the rotation
            // type's space requirement.
            #[linkme::distributed_slice(array_vis_bench_core::bench_registry::ALGORITHMS)]
            pub(super) static ENTRY: array_vis_bench_core::bench_registry::AlgorithmEntry =
                array_vis_bench_core::bench_registry::AlgorithmEntry {
                    name: NAME,
                    category: array_vis_bench_core::bench_registry::Category::Merge,
                    worst: <$merge as array_vis_bench_traits::composable::HasTimeBounds>::WORST,
                    best: <$merge as array_vis_bench_traits::composable::HasTimeBounds>::BEST,
                    average: <$merge as array_vis_bench_traits::composable::HasTimeBounds>::AVERAGE,
                    space: <$merge as array_vis_bench_traits::composable::HasSpace>::SPACE,
                    stable: <$merge as array_vis_bench_traits::composable::HasStability>::STABLE,
                    adaptive: false,
                    max_input_size: None,
                    run_with_input,
                    run_correctness,
                };

            #[ctor::ctor]
            fn register_path() {
                sort_registry_core::register_sort_path(
                    NAME,
                    <$merge as array_vis_bench_traits::composable::HasTimeBounds>::WORST.as_str(),
                    <$merge as array_vis_bench_traits::composable::HasStability>::STABLE,
                    &["merges", $family, ROT_NAME],
                );
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

use merge_sort_lib::auxiliary_merge::{AuxMerge, FullCopyAuxMerge, HalfCopyAuxMerge};

macro_rules! register_aux_merge {
    ($mod:ident, $merge:ty) => {
        mod $mod {
            use super::*;
            use sort_logger::{NoOpLogger, SortLogger};

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
                config: &array_vis_bench_core::bench_registry::RunConfig,
                logger: &mut dyn SortLogger<usize>,
            ) {
                array_vis_bench_core::bench_registry::run_merge_with_input(
                    input_name, config, merge_dyn, logger,
                );
            }

            fn run_correctness() {
                array_vis_bench_core::bench_registry::correctness::merge_battery(merge_noop, NAME);
            }

            #[linkme::distributed_slice(array_vis_bench_core::bench_registry::ALGORITHMS)]
            pub(super) static ENTRY: array_vis_bench_core::bench_registry::AlgorithmEntry =
                array_vis_bench_core::bench_registry::AlgorithmEntry {
                    name: NAME,
                    category: array_vis_bench_core::bench_registry::Category::Merge,
                    worst: <$merge as array_vis_bench_traits::composable::HasTimeBounds>::WORST,
                    best: <$merge as array_vis_bench_traits::composable::HasTimeBounds>::BEST,
                    average: <$merge as array_vis_bench_traits::composable::HasTimeBounds>::AVERAGE,
                    space: <$merge as array_vis_bench_traits::composable::HasSpace>::SPACE,
                    stable: <$merge as array_vis_bench_traits::composable::HasStability>::STABLE,
                    adaptive: false,
                    max_input_size: None,
                    run_with_input,
                    run_correctness,
                };

            #[ctor::ctor]
            fn register_path() {
                sort_registry_core::register_sort_path(
                    NAME,
                    <$merge as array_vis_bench_traits::composable::HasTimeBounds>::WORST.as_str(),
                    <$merge as array_vis_bench_traits::composable::HasStability>::STABLE,
                    &["merges", "auxiliary", VARIANT],
                );
            }

            }
    };
}

register_aux_merge!(aux_full, FullCopyAuxMerge);
register_aux_merge!(aux_half, HalfCopyAuxMerge);
