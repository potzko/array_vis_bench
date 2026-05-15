// Rotation algorithms ported from https://github.com/scandum/rotate (MIT, Igor van den Hoven)
//
// All functions take `arr` and `split_ind`: the index that becomes the new 0.
// After the call, arr[split_ind..] occupies the front and arr[..split_ind] the back.
// Equivalent to the C parameter `left` (left-block size).

use crate::traits::log_traits::SortLogger;

// ── Rotation trait ────────────────────────────────────────────────────────────

pub trait Rotation {
    const NAME: &'static str;

    /// Maximum auxiliary buffer this rotation needs for an input of length
    /// `n`. Returns 0 for fully in-place rotations. The caller is expected
    /// to pre-allocate `scratch_size(n)` elements and pass them as
    /// `scratch` to every call to [`Self::rotate`] during a single sort
    /// run — this lets the visualiser show a single aux array per run
    /// rather than one per rotation call.
    #[inline]
    fn scratch_size(_n: usize) -> usize {
        0
    }

    /// Rotate `arr` so that `arr[split_ind..]` becomes the new prefix and
    /// `arr[..split_ind]` becomes the new suffix.
    ///
    /// `scratch` is the pre-allocated scratch buffer described by
    /// [`Self::scratch_size`]. Rotations with `scratch_size = 0` may
    /// receive an empty slice and should ignore it.
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        split_ind: usize,
        scratch: &mut [T],
        logger: &mut U,
    );
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
    let mut buf = logger.create_aux_arr_t(left);
    buf_rotate_left_using(arr, left, &mut buf, logger);
    logger.free_aux_arr_t(&buf);
}

pub(super) fn buf_rotate_right<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    left: usize,
    logger: &mut U,
) {
    let right = arr.len() - left;
    let mut buf = logger.create_aux_arr_t(right);
    buf_rotate_right_using(arr, left, &mut buf, logger);
    logger.free_aux_arr_t(&buf);
}

/// Buffered left-rotation using a caller-provided scratch buffer (must
/// have length `>= left`). Useful when the caller already owns a
/// pre-registered scratch slice (e.g. threaded through a sort).
pub(super) fn buf_rotate_left_using<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    left: usize,
    scratch: &mut [T],
    logger: &mut U,
) {
    let right = arr.len() - left;
    logger.copy_range(arr, 0, scratch, 0, left);
    for i in 0..right {
        let v = arr[left + i];
        logger.write_data(arr, i, v);
    }
    logger.copy_range(scratch, 0, arr, right, left);
}

/// Buffered right-rotation using a caller-provided scratch buffer (must
/// have length `>= arr.len() - left`).
pub(super) fn buf_rotate_right_using<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    left: usize,
    scratch: &mut [T],
    logger: &mut U,
) {
    let right = arr.len() - left;
    logger.copy_range(arr, left, scratch, 0, right);
    for i in (0..left).rev() {
        let v = arr[i];
        logger.write_data(arr, right + i, v);
    }
    logger.copy_range(scratch, 0, arr, 0, right);
}

/// Left-rotate `arr` by 1: `[a, b, c, d]` → `[b, c, d, a]`. Truly in-place
/// — no aux array — using a single saved value and an `n-1` element shift.
pub(super) fn unit_rotate_left<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) {
    let n = arr.len();
    if n < 2 {
        return;
    }
    let saved = arr[0];
    for i in 0..n - 1 {
        logger.write(arr, i, i + 1);
    }
    logger.write_data(arr, n - 1, saved);
}

/// Right-rotate `arr` by 1: `[a, b, c, d]` → `[d, a, b, c]`. Truly in-place.
pub(super) fn unit_rotate_right<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) {
    let n = arr.len();
    if n < 2 {
        return;
    }
    let saved = arr[n - 1];
    for i in (1..n).rev() {
        logger.write(arr, i, i - 1);
    }
    logger.write_data(arr, 0, saved);
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

/// Run a rotation as a one-shot: allocate the rotation's scratch buffer,
/// run the rotation, and free the buffer. Used by the standalone rotation
/// entries (registered under `/rotations/`) and the per-merge entries
/// (registered under `/merges/...`). A repeated caller (a merge sort) is
/// expected to allocate the scratch once at its top level and call
/// `R::rotate` directly, passing the registered slice down through every
/// rotation invocation.
pub fn run_rotation<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    split_ind: usize,
    logger: &mut U,
) {
    let scratch_size = R::scratch_size(arr.len());
    if scratch_size == 0 {
        R::rotate(arr, split_ind, &mut [], logger);
    } else {
        let mut scratch = logger.create_aux_arr_t(scratch_size);
        R::rotate(arr, split_ind, &mut scratch, logger);
        logger.free_aux_arr_t(&scratch);
    }
}

// (The old `check_rotation` test helper has been retired in favour of
// `bench_registry::correctness::rotation_battery`, which is what the
// `register_rotation!` macro now calls from each rotation's per-entry
// `run_correctness`.)
