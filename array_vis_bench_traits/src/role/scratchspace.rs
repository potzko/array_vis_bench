//! `Scratchspace` — a set of contracts over one or more sequential lists,
//! realised through a small auxiliary buffer.
//!
//! A scratchspace is *bounded borrowable storage*: it lets an algorithm relocate
//! runs of elements without `O(n)` external memory. The **contract** is the
//! operation set below and the invariant they jointly uphold; an
//! **implementation** is a specific way of realising those operations together.
//!
//! Aux-buffer convention (same as [`Rotation`](crate::role::Rotation)):
//! [`aux_size`](Scratchspace::aux_size) declares how many auxiliary cells the
//! implementation needs for an `n`-element list (`0` = fully in-place). The
//! caller allocates that once via the logger and threads the `aux` slice into
//! **every** operation; an op uses as much of it as it needs (some ignore it).
//!
//! Invariant every operation upholds: **conservation** — the multiset of
//! `list ∪ aux` is preserved; nothing is created, dropped, or duplicated. Each
//! self-contained op (`block_swap` / `relocate` / `roll`) leaves the list a
//! permutation of its input with `aux` reusable; `park`/`place` are the borrow
//! pair (`aux` carries the parked run between them).

use sort_logger::SortLogger;

pub trait Scratchspace {
    /// Short label used in the generated variant name (`scratchspace<full>`).
    const NAME: &'static str;

    /// Auxiliary cells to allocate for operating over an `n`-element list.
    /// `0` = fully in-place. Caller allocates once and threads into every op.
    fn aux_size(n: usize) -> usize;

    /// Ready the scratchspace over `list` before use (default no-op). A stateful
    /// backing may prime `aux` / extract a region here — mirrors
    /// `HeapAlgorithm::new_state`.
    fn acquire<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        _list: &mut [T],
        _aux: &mut [T],
        _log: &mut U,
    ) {
    }

    /// Reclaim the scratchspace after use (default no-op); the dual of `acquire`.
    fn release<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        _list: &mut [T],
        _aux: &mut [T],
        _log: &mut U,
    ) {
    }

    /// Copy `list[at..at+len]` into the scratchspace (`len <= aux.len()`), so the
    /// caller may overwrite those cells; [`place`](Scratchspace::place) returns them.
    fn park<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        list: &mut [T],
        at: usize,
        len: usize,
        aux: &mut [T],
        log: &mut U,
    ) {
        log.copy_range(list, at, aux, 0, len);
    }

    /// Write the run parked by the matching [`park`](Scratchspace::park) to
    /// `list[at..at+len]`.
    fn place<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        list: &mut [T],
        at: usize,
        len: usize,
        aux: &mut [T],
        log: &mut U,
    ) {
        log.copy_range(aux, 0, list, at, len);
    }

    /// Swap two equal, disjoint regions `list[a..a+len]` and `list[b..b+len]`.
    /// Default is in-place element swaps (ignores `aux`).
    fn block_swap<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        list: &mut [T],
        a: usize,
        b: usize,
        len: usize,
        _aux: &mut [T],
        log: &mut U,
    ) {
        log.block_swap(list, a, b, len);
    }

    /// Move `list[from..from+len]` so it begins at `to`, sliding the intervening
    /// elements to fill the gap. Parks the block when it fits (`len <= aux.len()`),
    /// otherwise falls back to a rotation of the enclosing range.
    fn relocate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        list: &mut [T],
        from: usize,
        to: usize,
        len: usize,
        aux: &mut [T],
        log: &mut U,
    ) {
        if from == to || len == 0 {
            return;
        }
        if len <= aux.len() {
            Self::park(list, from, len, aux, log);
            if to < from {
                // slide list[to..from] right by len (high→low, no clobber)
                for i in (0..(from - to)).rev() {
                    log.write(list, to + len + i, to + i);
                }
            } else {
                // slide list[from+len..to+len] left by len (low→high)
                for i in 0..(to - from) {
                    log.write(list, from + i, from + len + i);
                }
            }
            Self::place(list, to, len, aux, log);
        } else {
            // no room to park → rotate the enclosing range instead
            let (lo, hi, k) = if to < from {
                (to, from + len, from - to) // left-rotate by (from-to)
            } else {
                (from, to + len, len) // left-rotate by len
            };
            Self::roll(list, lo, hi, k, aux, log);
        }
    }

    /// Left-rotate `list[lo..hi]` by `k`. Default is three reversals (in place,
    /// ignores `aux`).
    fn roll<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        list: &mut [T],
        lo: usize,
        hi: usize,
        k: usize,
        _aux: &mut [T],
        log: &mut U,
    ) {
        let n = hi - lo;
        if n == 0 {
            return;
        }
        let k = k % n;
        if k == 0 {
            return;
        }
        reverse_range(list, lo, lo + k, log);
        reverse_range(list, lo + k, hi, log);
        reverse_range(list, lo, hi, log);
    }

}

/// A **runnable workflow** over a scratchspace — the emittable unit of the
/// `Category::Scratchspace` kind. Each workflow is a scripted routine that
/// borrows a specific backing `S: Scratchspace` (via [`with_scratch`]) and drives
/// its operations; different workflows show different usage patterns (a guided
/// tour, repeated parking, park/place loops, targeted relocations…). Kept
/// separate from [`Scratchspace`] so *backing* (aux strategy) and *workflow*
/// (what you do with it) are independent axes the catalog cross-products.
pub trait ScratchspaceRun {
    /// Run the workflow over `list`, allocating/freeing its own auxiliary.
    fn run<T: Ord + Copy, U: ?Sized + SortLogger<T>>(list: &mut [T], log: &mut U);
}

/// Allocate `S`'s `aux_size` auxiliary via the logger (empty slice when `0`),
/// hand `(list, aux, log)` to `body`, then free it. The shared alloc/free
/// lifecycle every workflow and scratchspace-backed algorithm reuses — the same
/// shape as [`run_rotation`](crate::role::rotation::run_rotation).
pub fn with_scratch<S: Scratchspace, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    list: &mut [T],
    log: &mut U,
    body: fn(&mut [T], &mut [T], &mut U),
) {
    let sz = S::aux_size(list.len());
    if sz == 0 {
        body(list, &mut [], log);
    } else {
        let mut aux = log.create_aux_arr_t(sz);
        body(list, &mut aux, log);
        log.free_aux_arr_t(&aux);
    }
}

/// Reverse `list[lo..hi]` via logged single-array swaps (so the whole list stays
/// one array in the visualiser, unlike `logger.reverse` on a sub-slice).
fn reverse_range<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    list: &mut [T],
    lo: usize,
    hi: usize,
    log: &mut U,
) {
    if hi <= lo {
        return;
    }
    let (mut i, mut j) = (lo, hi - 1);
    while i < j {
        log.swap(list, i, j);
        i += 1;
        j -= 1;
    }
}

