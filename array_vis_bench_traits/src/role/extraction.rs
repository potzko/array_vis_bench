//! `Extraction`, the carving role of a block merge sort: how `want` distinct
//! elements are found in a region and gathered, sorted, at its front, each the
//! first occurrence of its value, the rest keeping their order among equals.
//! The scan in `keys` is the plain filler; the others prepare the region so
//! that the scan has less to do.

use crate::Rotation;
use sort_logger::SortLogger;

pub trait Extraction {
    /// Short label used in the generated variant name.
    const NAME: &'static str;

    /// Gather up to `want` distinct elements of `arr[lo..hi]` at its front,
    /// sorted, and return how many were found. Each is the first occurrence of
    /// its value, so an element that stays is never ahead of an equal one that
    /// left, and the elements that stay keep their order among equals. `scratch`
    /// is the rotation's, `R::scratch_size` cells or more.
    fn extract<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        want: usize,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize;
}
