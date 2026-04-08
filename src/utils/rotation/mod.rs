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
        fn __rotate_entry_fn(
            arr: &mut [usize],
            split: usize,
            logger: &mut dyn crate::traits::log_traits::SortLogger<usize>,
        ) {
            <$rot as crate::utils::rotation::Rotation>::rotate(arr, split, logger)
        }

        #[linkme::distributed_slice(crate::utils::rotation::ROTATIONS)]
        static _ROTATION_ENTRY: crate::utils::rotation::RotationEntry =
            crate::utils::rotation::RotationEntry {
                name: <$rot as crate::utils::rotation::Rotation>::NAME,
                rotate_fn: __rotate_entry_fn,
            };

        #[cfg(test)]
        mod rotation_test {
            #[test]
            fn correctness() {
                crate::utils::rotation::test_helpers::check_rotation(&super::_ROTATION_ENTRY);
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

// ── Test helpers (used by register_rotation! macro) ─────────────────────────

#[cfg(test)]
pub(crate) mod test_helpers {
    use super::RotationEntry;
    use crate::traits::log_traits::NoOpLogger;

    fn check(entry: &RotationEntry, n: usize, split: usize) {
        let mut arr: Vec<usize> = (0..n).collect();
        let expected: Vec<usize> = (split..n).chain(0..split).collect();
        (entry.rotate_fn)(&mut arr, split, &mut NoOpLogger);
        assert_eq!(
            arr, expected,
            "{}: rotate({}, {}) failed",
            entry.name, n, split
        );
    }

    pub fn check_rotation(entry: &RotationEntry) {
        check(entry, 0, 0);
        check(entry, 1, 0);
        check(entry, 1, 1);
        check(entry, 2, 0);
        check(entry, 2, 1);
        check(entry, 2, 2);
        for n in 3..=16 {
            for split in 0..=n {
                check(entry, n, split);
            }
        }
        for n in [32, 100, 127, 128, 255, 1000] {
            check(entry, n, 1);
            check(entry, n, n - 1);
            check(entry, n, n / 2);
            check(entry, n, n / 3);
            check(entry, n, n * 2 / 3);
        }
    }
}
