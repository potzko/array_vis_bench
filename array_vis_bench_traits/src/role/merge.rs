//! `RotationMerge`, the merge role: merge two adjacent sorted runs in place.
//! `run_rotation_merge` is the one-shot that owns the scratch buffer, the merge
//! analogue of `rotation::run_rotation`.

use sort_logger::SortLogger;

/// Strategy for merging two adjacent sorted slices in-place using rotation.
///
/// `scratch_size(n)` returns the size of an aux buffer the caller should
/// pre-allocate; `merge` then receives that buffer and forwards it to
/// every `R::rotate` invocation so the visualiser shows a single aux
/// array per sort run rather than one per rotation.
pub trait RotationMerge {
    /// Required scratch size for merging an array of length `n`. Delegates
    /// to the underlying rotation by default.
    fn scratch_size(n: usize) -> usize;

    /// Merge `arr[..mid]` and `arr[mid..]` in-place, using `scratch` as
    /// caller-owned aux memory passed down to the rotation.
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        mid: usize,
        scratch: &mut [T],
        logger: &mut U,
    );
}

/// Run a single [`RotationMerge`] as a one-shot: allocate the merge's scratch
/// buffer, merge `arr[..mid]` with `arr[mid..]`, then free the buffer. The
/// standalone `Category::Merge` entries use this. A repeated caller (a merge
/// sort) allocates the scratch once at its top level instead.
pub fn run_rotation_merge<M, T, U>(arr: &mut [T], mid: usize, logger: &mut U)
where
    M: RotationMerge,
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
{
    let scratch_size = M::scratch_size(arr.len());
    if scratch_size == 0 {
        M::merge(arr, mid, &mut [], logger);
    } else {
        let mut scratch = logger.create_aux_arr_t(scratch_size);
        M::merge(arr, mid, &mut scratch, logger);
        logger.free_aux_arr_t(&scratch);
    }
}
