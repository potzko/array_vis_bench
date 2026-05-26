//! `Rotation` — rotation algorithm role + the shared in-place helpers
//! every rotation impl reaches for.
//!
//! Convention: every rotation takes `arr` and `split_ind` where
//! `split_ind` is the index that becomes the new 0. After the call,
//! `arr[split_ind..]` occupies the front and `arr[..split_ind]` the
//! back. Equivalent to the C parameter `left` in scandum/rotate.
//!
//! Leaf crates (`rotation_reversal`, `rotation_gries_mills`, …) impl
//! `Rotation` and reach for the helpers below; the wiring crate
//! (`array_vis_bench`) handles registration into the
//! `ROTATIONS` distributed slice and the `ALGORITHMS` entry table.

use sort_logger::SortLogger;

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

// ── Shared helpers ───────────────────────────────────────────────────────────
//
// `pub` so leaf crates can reach for them. Previously `pub(super)` inside
// `array_vis_bench/src/utils/rotation/mod.rs`; visibility widened for
// the per-crate split.

#[inline(always)]
pub fn reverse<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    logger.reverse(arr);
}

#[inline(always)]
pub fn forward_block_swap<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    s1: usize,
    s2: usize,
    n: usize,
    logger: &mut U,
) {
    logger.block_swap(arr, s1, s2, n);
}

#[inline(always)]
pub fn backward_block_swap<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
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

pub fn buf_rotate_left<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    left: usize,
    logger: &mut U,
) {
    let mut buf = logger.create_aux_arr_t(left);
    buf_rotate_left_using(arr, left, &mut buf, logger);
    logger.free_aux_arr_t(&buf);
}

pub fn buf_rotate_right<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
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
pub fn buf_rotate_left_using<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
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
pub fn buf_rotate_right_using<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
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
pub fn unit_rotate_left<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
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
pub fn unit_rotate_right<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
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

pub fn gcd(mut a: usize, mut b: usize) -> usize {
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
