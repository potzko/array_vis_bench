//! `StablePartition`, the stable partition role: the elements of a region
//! below a pivot value brought to its front and the rest after them, each side
//! keeping its order. The partition extraction of a block merge sort selects
//! its sample with one, and a stable quick sort would partition with one. A
//! filler rotates with the `Rotation` it is handed and may use that rotation's
//! scratch as a buffer.

use crate::Complexity;
use crate::Rotation;
use sort_logger::SortLogger;

pub trait StablePartition {
    /// Short label used in the generated variant name.
    const NAME: &'static str;

    /// Worst case moves of a partition of `n` elements without scratch.
    const MOVES: Complexity;

    /// Move the elements of `arr[lo..hi]` below `pivot` to its front, each side
    /// keeping its order, and return where the rest begins. `scratch` is the
    /// rotation's, `R::scratch_size` cells or more.
    fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize;
}
