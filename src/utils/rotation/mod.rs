// Rotation algorithms ported from https://github.com/scandum/rotate (MIT, Igor van den Hoven)
//
// All functions take `arr` and `split_ind`: the index that becomes the new 0.
// After the call, arr[split_ind..] occupies the front and arr[..split_ind] the back.
// Equivalent to the C parameter `left` (left-block size).

use crate::traits::log_traits::SortLogger;

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

// ── Rotation trait ────────────────────────────────────────────────────────────

pub trait Rotation {
    const NAME: &'static str;
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], split_ind: usize, logger: &mut U);
}

// ── Shared helpers ────────────────────────────────────────────────────────────

#[inline(always)]
pub fn reverse<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let n = arr.len();
    let mut i = 0;
    let mut ii = n.saturating_sub(1);
    while i < ii {
        let tmp = arr[i];
        logger.write_data(arr, i, arr[ii]);
        logger.write_data(arr, ii, tmp);
        i += 1;
        ii -= 1;
    }
}

#[inline(always)]
pub(super) fn forward_block_swap<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    s1: usize,
    s2: usize,
    n: usize,
    logger: &mut U,
) {
    for i in 0..n {
        let tmp = arr[s1 + i];
        logger.write_data(arr, s1 + i, arr[s2 + i]);
        logger.write_data(arr, s2 + i, tmp);
    }
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
        let tmp = arr[s1 + i];
        logger.write_data(arr, s1 + i, arr[s2 + i]);
        logger.write_data(arr, s2 + i, tmp);
    }
}

pub(super) fn buf_rotate_left<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    left: usize,
    logger: &mut U,
) {
    let right = arr.len() - left;
    let mut buf = logger.create_aux_arr_t(left);
    for i in 0..left {
        logger.write_accross(arr, i, &mut buf, i);
    }
    for i in 0..right {
        let v = arr[left + i];
        logger.write_data(arr, i, v);
    }
    for i in 0..left {
        logger.write_data(arr, right + i, buf[i]);
    }
    logger.free_aux_arr_t(&buf);
}

pub(super) fn buf_rotate_right<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    left: usize,
    logger: &mut U,
) {
    let right = arr.len() - left;
    let mut buf = logger.create_aux_arr_t(right);
    for i in 0..right {
        logger.write_accross(arr, left + i, &mut buf, i);
    }
    for i in (0..left).rev() {
        let v = arr[i];
        logger.write_data(arr, right + i, v);
    }
    for i in 0..right {
        logger.write_data(arr, i, buf[i]);
    }
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::log_traits::NoOpLogger;

    fn check<R: Rotation>(n: usize, split: usize) {
        let mut arr: Vec<usize> = (0..n).collect();
        let expected: Vec<usize> = (split..n).chain(0..split).collect();
        R::rotate(&mut arr, split, &mut NoOpLogger);
        assert_eq!(
            arr, expected,
            "{}: rotate({}, {}) failed",
            std::any::type_name::<R>(), n, split
        );
    }

    fn check_all<R: Rotation>() {
        check::<R>(0, 0);
        check::<R>(1, 0);
        check::<R>(1, 1);
        check::<R>(2, 0);
        check::<R>(2, 1);
        check::<R>(2, 2);
        for n in 3..=16 {
            for split in 0..=n {
                check::<R>(n, split);
            }
        }
        for n in [32, 100, 127, 128, 255, 1000] {
            check::<R>(n, 1);
            check::<R>(n, n - 1);
            check::<R>(n, n / 2);
            check::<R>(n, n / 3);
            check::<R>(n, n * 2 / 3);
        }
    }

    #[test] fn reversal()   { check_all::<ReversalRotation>(); }
    #[test] fn auxiliary()  { check_all::<AuxiliaryRotation>(); }
    #[test] fn bridge()     { check_all::<BridgeRotation>(); }
    #[test] fn contrev()    { check_all::<ContrevRotation>(); }
    #[test] fn trinity()    { check_all::<TrinityRotation>(); }
    #[test] fn griesmills() { check_all::<GriesMillsRotation>(); }
    #[test] fn grail()      { check_all::<GrailRotation>(); }
    #[test] fn piston()     { check_all::<PistonRotation>(); }
    #[test] fn helix()      { check_all::<HelixRotation>(); }
    #[test] fn drill()      { check_all::<DrillRotation>(); }
    #[test] fn juggling()   { check_all::<JugglingRotation>(); }
}
