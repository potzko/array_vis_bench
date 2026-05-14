// Rotation algorithms ported from https://github.com/scandum/rotate (MIT, Igor van den Hoven)
//
// All functions take `arr` and `split_ind`: the index that becomes the new 0.
// After the call, arr[split_ind..] occupies the front and arr[..split_ind] the back.
// Equivalent to the C parameter `left` (left-block size).

use crate::traits::log_traits::SortLogger;

// ── Rotation trait ────────────────────────────────────────────────────────────

pub trait Rotation {
    const NAME: &'static str;
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], split_ind: usize, logger: &mut U);
}

// ── Rotation registry ────────────────────────────────────────────────────────

pub struct RotationEntry {
    pub name: &'static str,
    pub rotate_fn: fn(&mut [usize], usize, &mut dyn SortLogger<usize>),
}

#[linkme::distributed_slice]
pub static ROTATIONS: [RotationEntry] = [..];

macro_rules! register_rotation {
    ($rot:ty) => {
        const _ROTATION_NAME: &str = const_format::concatcp!(
            "rotation: ",
            <$rot as crate::utils::rotation::Rotation>::NAME,
        );

        // `rotate_fn` is the type-erased dyn-logger dispatcher used by
        // the existing `ROTATIONS` slice (merge sorts iterate it to look
        // a rotation up by name). Same body works as the entry point
        // for `run_rotation_with_input` since both want a dyn logger.
        fn __rotate_dyn(
            arr: &mut [usize],
            split: usize,
            logger: &mut dyn crate::traits::log_traits::SortLogger<usize>,
        ) {
            <$rot as crate::utils::rotation::Rotation>::rotate(arr, split, logger)
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
            <$rot as crate::utils::rotation::Rotation>::rotate(arr, split, logger)
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

        #[linkme::distributed_slice(crate::bench_registry::ALGORITHMS)]
        static _ALGO_ENTRY: crate::bench_registry::AlgorithmEntry =
            crate::bench_registry::AlgorithmEntry {
                name: _ROTATION_NAME,
                category: crate::bench_registry::Category::Rotation,
                big_o: "O(N)",
                stable: false,
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
    };
}
#[allow(unused_imports)]
pub(crate) use register_rotation;

// ── Submodules ───────────────────────────────────────────────────────────────

pub mod auxiliary;
pub mod bridge;
pub mod contrev;
pub mod drill;
pub mod grail;
pub mod gries_mills;
pub mod helix;
pub mod juggling;
pub mod piston;
pub mod reversal;
pub mod trinity;

pub use auxiliary::AuxiliaryRotation;
pub use bridge::BridgeRotation;
pub use contrev::ContrevRotation;
pub use drill::DrillRotation;
pub use grail::GrailRotation;
pub use gries_mills::GriesMillsRotation;
pub use helix::HelixRotation;
pub use juggling::JugglingRotation;
pub use piston::PistonRotation;
pub use reversal::ReversalRotation;
pub use trinity::TrinityRotation;

combo_codegen::component!(Rotation, ReversalRotation,   "reversal");
combo_codegen::component!(Rotation, AuxiliaryRotation,  "auxiliary");
combo_codegen::component!(Rotation, BridgeRotation,     "bridge");
combo_codegen::component!(Rotation, ContrevRotation,    "contrev");
combo_codegen::component!(Rotation, TrinityRotation,    "trinity");
combo_codegen::component!(Rotation, GriesMillsRotation, "gries-mills");
combo_codegen::component!(Rotation, GrailRotation,      "grail");
combo_codegen::component!(Rotation, PistonRotation,     "piston");
combo_codegen::component!(Rotation, HelixRotation,      "helix");
combo_codegen::component!(Rotation, DrillRotation,      "drill");
combo_codegen::component!(Rotation, JugglingRotation,   "juggling");

// ── Shared helpers ────────────────────────────────────────────────────────────

#[inline(always)]
pub fn reverse<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    logger.reverse(arr);
}

#[inline(always)]
pub(super) fn forward_block_swap<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    s1: usize,
    s2: usize,
    n: usize,
    logger: &mut U,
) {
    logger.block_swap(arr, s1, s2, n);
}

#[inline(always)]
pub(super) fn backward_block_swap<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    s1: usize,
    s2: usize,
    n: usize,
    logger: &mut U,
) {
    for i in (0..n).rev() {
        logger.swap(arr, s1 + i, s2 + i);
    }
}

pub(super) fn buf_rotate_left<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    left: usize,
    logger: &mut U,
) {
    let right = arr.len() - left;
    let mut buf = logger.create_aux_arr_t(left);
    logger.copy_range(arr, 0, &mut buf, 0, left);
    for i in 0..right {
        let v = arr[left + i];
        logger.write_data(arr, i, v);
    }
    logger.copy_range(&buf, 0, arr, right, left);
    logger.free_aux_arr_t(&buf);
}

pub(super) fn buf_rotate_right<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    left: usize,
    logger: &mut U,
) {
    let right = arr.len() - left;
    let mut buf = logger.create_aux_arr_t(right);
    logger.copy_range(arr, left, &mut buf, 0, right);
    for i in (0..left).rev() {
        let v = arr[i];
        logger.write_data(arr, right + i, v);
    }
    logger.copy_range(&buf, 0, arr, 0, right);
    logger.free_aux_arr_t(&buf);
}

pub(super) fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

// ── Convenience generic dispatcher ───────────────────────────────────────────

pub fn rotate<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    split_ind: usize,
    logger: &mut U,
) {
    R::rotate(arr, split_ind, logger);
}

// (The old `check_rotation` test helper has been retired in favour of
// `bench_registry::correctness::rotation_battery`, which is what the
// `register_rotation!` macro now calls from each rotation's per-entry
// `run_correctness`.)
