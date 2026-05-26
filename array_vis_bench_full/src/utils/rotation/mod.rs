// Rotation algorithms ported from https://github.com/scandum/rotate (MIT, Igor van den Hoven)
//
// All functions take `arr` and `split_ind`: the index that becomes the new 0.
// After the call, arr[split_ind..] occupies the front and arr[..split_ind] the back.
// Equivalent to the C parameter `left` (left-block size).

use crate::traits::log_traits::SortLogger;

// `Rotation` lives in `array_vis_bench_traits` so leaf-component crates
// (`rotation_reversal`, …) can implement it without depending on the
// full `array_vis_bench` tree. Re-exported here so every existing
// `crate::utils::rotation::Rotation` path keeps resolving.
pub use array_vis_bench_traits::role::rotation::{
    backward_block_swap, buf_rotate_left, buf_rotate_left_using, buf_rotate_right,
    buf_rotate_right_using, forward_block_swap, gcd, reverse, run_rotation, unit_rotate_left,
    unit_rotate_right, Rotation,
};

// ── Rotation registry ────────────────────────────────────────────────────────

pub struct RotationEntry {
    pub name: &'static str,
    pub rotate_fn: fn(&mut [usize], usize, &mut dyn SortLogger<usize>),
}

#[linkme::distributed_slice]
pub static ROTATIONS: [RotationEntry] = [..];

// Each invocation wraps everything in a per-rotation private module so
// multiple `register_rotation!` calls can live in one file without their
// generated items (`_ROTATION_ENTRY`, `_ALGO_ENTRY`, `__rotate_dyn`, the
// inner `mod rotation_test`) colliding. The `$mod` identifier is just a
// unique slug per invocation.
macro_rules! register_rotation {
    ($mod:ident, $rot:ty) => {
        mod $mod {
            use super::*;

            const _ROTATION_NAME: &str = const_format::concatcp!(
                "rotation: ",
                <$rot as crate::utils::rotation::Rotation>::NAME,
            );

            // `rotate_fn` is the type-erased dyn-logger dispatcher used by
            // the existing `ROTATIONS` slice (merge sorts iterate it to
            // look a rotation up by name). Same body works as the entry
            // point for `run_rotation_with_input` since both want a dyn
            // logger.
            fn __rotate_dyn(
                arr: &mut [usize],
                split: usize,
                logger: &mut dyn crate::traits::log_traits::SortLogger<usize>,
            ) {
                crate::utils::rotation::run_rotation::<$rot, usize, _>(arr, split, logger)
            }

            // NoOp-logger variant used by the correctness battery — same
            // body, different logger type. The battery wants a concrete
            // `NoOpLogger` so the rotation doesn't pay dyn-dispatch cost
            // while the tests run thousands of cases.
            fn __rotate_noop(
                arr: &mut [usize],
                split: usize,
                logger: &mut crate::traits::log_traits::NoOpLogger,
            ) {
                crate::utils::rotation::run_rotation::<$rot, usize, _>(arr, split, logger)
            }

            #[linkme::distributed_slice(crate::utils::rotation::ROTATIONS)]
            static _ROTATION_ENTRY: crate::utils::rotation::RotationEntry =
                crate::utils::rotation::RotationEntry {
                    name: <$rot as crate::utils::rotation::Rotation>::NAME,
                    rotate_fn: __rotate_dyn,
                };

            // Unified algorithm entry: rotations appear under `/rotations/`
            // in the menu and are testable / visualisable through the same
            // pipeline as sorts.
            fn __run_with_input(
                input_name: &str,
                config: &crate::bench_registry::RunConfig,
                logger: &mut dyn crate::traits::log_traits::SortLogger<usize>,
            ) {
                crate::bench_registry::run_rotation_with_input(
                    input_name, config, __rotate_dyn, logger,
                );
            }
            fn __run_correctness() {
                crate::bench_registry::correctness::rotation_battery(
                    __rotate_noop,
                    _ROTATION_NAME,
                );
            }

            // `HasTimeBounds`, `HasSpace`, `HasStability` impls all live
            // in the leaf crate alongside the type — the orphan rule
            // forbids impl-ing a foreign trait for a foreign type from a
            // third crate. The leaf provides uniform values for the
            // rotation family (N1 time, false stable; SPACE varies per
            // rotation).

            #[linkme::distributed_slice(crate::bench_registry::ALGORITHMS)]
            static _ALGO_ENTRY: crate::bench_registry::AlgorithmEntry =
                crate::bench_registry::AlgorithmEntry {
                    name: _ROTATION_NAME,
                    category: crate::bench_registry::Category::Rotation,
                    worst: <$rot as crate::traits::composable::HasTimeBounds>::WORST,
                    best: <$rot as crate::traits::composable::HasTimeBounds>::BEST,
                    average: <$rot as crate::traits::composable::HasTimeBounds>::AVERAGE,
                    space: <$rot as crate::traits::composable::HasSpace>::SPACE,
                    stable: <$rot as crate::traits::composable::HasStability>::STABLE,
                    adaptive: false,
                    max_input_size: None,
                    run_with_input: __run_with_input,
                    run_correctness: __run_correctness,
                };

            #[ctor::ctor]
            fn __register_path() {
                sort_registry_core::register_sort_path(
                    _ROTATION_NAME,
                    "O(N)",
                    false,
                    &["rotations", <$rot as crate::utils::rotation::Rotation>::NAME],
                );
            }

            #[cfg(test)]
            mod rotation_test {
                #[test]
                fn correctness() {
                    crate::bench_registry::test_helpers::check_sort_subprocess_assert(
                        &super::_ALGO_ENTRY,
                        crate::bench_registry::test_helpers::DEFAULT_TIMEOUT,
                    );
                }
            }
        }
    };
}
#[allow(unused_imports)]
pub(crate) use register_rotation;

// ── Leaf crate re-exports ────────────────────────────────────────────────────
//
// Each rotation is now its own leaf crate (`rotation_reversal`,
// `rotation_auxiliary`, …). The type + `Rotation` impl + `HasSpace` impl
// live there. Component metadata for the family! cross-product is in each
// leaf's Cargo.toml. The standalone-algorithm registration (the
// `ROTATIONS` distributed slice + the `/rotations/` menu entry +
// `ALGORITHMS` entry + `HasTimeBounds` / `HasStability` impls) comes
// from `register_rotation!` invocations below — kept central so each
// leaf stays a pure component crate.

pub use rotation_auxiliary::AuxiliaryRotation;
pub use rotation_bridge::BridgeRotation;
pub use rotation_contrev::ContrevRotation;
pub use rotation_drill::DrillRotation;
pub use rotation_grail::GrailRotation;
pub use rotation_gries_mills::GriesMillsRotation;
pub use rotation_helix::HelixRotation;
pub use rotation_juggling::JugglingRotation;
pub use rotation_piston::PistonRotation;
pub use rotation_reversal::ReversalRotation;
pub use rotation_trinity::TrinityRotation;

register_rotation!(_reg_reversal,    ReversalRotation);
register_rotation!(_reg_auxiliary,   AuxiliaryRotation);
register_rotation!(_reg_bridge,      BridgeRotation);
register_rotation!(_reg_contrev,     ContrevRotation);
register_rotation!(_reg_trinity,     TrinityRotation);
register_rotation!(_reg_gries_mills, GriesMillsRotation);
register_rotation!(_reg_grail,       GrailRotation);
register_rotation!(_reg_piston,      PistonRotation);
register_rotation!(_reg_helix,       HelixRotation);
register_rotation!(_reg_drill,       DrillRotation);
register_rotation!(_reg_juggling,    JugglingRotation);


// (The old `check_rotation` test helper has been retired in favour of
// `bench_registry::correctness::rotation_battery`, which is what the
// `register_rotation!` macro now calls from each rotation's per-entry
// `run_correctness`. The shared helpers and `run_rotation` dispatcher
// live in `array_vis_bench_traits::role::rotation` — re-exported above.)
