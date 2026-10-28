//! Scratchspace **workflows** (runnable routines over the ops) + a scratchspace
//! -backed sort. Each workflow is generic over the backing `S: Scratchspace`, so
//! the catalog cross-products workflow × backing. Every workflow's net effect is
//! identity (its correctness invariant) while it drives a distinct usage pattern
//! through the logger; the nightmare sort actually sorts.

use core::marker::PhantomData;

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::role::scratchspace::with_scratch;
use array_vis_bench_traits::role::{Scratchspace, ScratchspaceRun};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

use crate::bufferless::bufferless_merge;

// ── Workflow: a guided tour of every operation, each paired with its inverse ──

/// acquire → park/place → block_swap → relocate → roll → release. Net identity.
pub struct Exercise<S>(PhantomData<S>);
impl<S: Scratchspace> ScratchspaceRun for Exercise<S> {
    fn run<T: Ord + Copy, U: ?Sized + SortLogger<T>>(list: &mut [T], log: &mut U) {
        with_scratch::<S, T, U>(list, log, exercise_body::<S, T, U>);
    }
}
fn exercise_body<S: Scratchspace, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    list: &mut [T],
    aux: &mut [T],
    log: &mut U,
) {
    let n = list.len();
    if n < 8 {
        return;
    }
    S::acquire(list, aux, log);
    let cap = aux.len();
    let blk = (n / 8).max(1);
    if cap > 0 {
        let len = blk.min(cap);
        S::park(list, 0, len, aux, log);
        S::place(list, 0, len, aux, log);
    }
    if 2 * blk <= n {
        S::block_swap(list, 0, n - blk, blk, aux, log);
        S::block_swap(list, 0, n - blk, blk, aux, log);
    }
    {
        let (len, from, to) = (blk, n / 2, n / 4);
        if to + len <= from && from + len <= n {
            S::relocate(list, from, to, len, aux, log);
            S::relocate(list, to, from, len, aux, log);
        }
    }
    {
        let k = blk.min(n - 1);
        S::roll(list, 0, n, k, aux, log);
        S::roll(list, 0, n, n - k, aux, log);
    }
    S::release(list, aux, log);
}

// ── Workflow: park up to 100 successive windows WITHOUT placing them back ──

/// Repeatedly `park` windows of the list into the buffer without `place` — the
/// buffer reloads over and over while the list stays put (park is a copy). Shows
/// the aux churning. (No-op for a zero-aux backing: there's nowhere to park.)
pub struct ParkSweep<S>(PhantomData<S>);
impl<S: Scratchspace> ScratchspaceRun for ParkSweep<S> {
    fn run<T: Ord + Copy, U: ?Sized + SortLogger<T>>(list: &mut [T], log: &mut U) {
        with_scratch::<S, T, U>(list, log, park_sweep_body::<S, T, U>);
    }
}
fn park_sweep_body<S: Scratchspace, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    list: &mut [T],
    aux: &mut [T],
    log: &mut U,
) {
    let (n, cap) = (list.len(), aux.len());
    if cap == 0 || n == 0 {
        return;
    }
    let len = cap.min(n);
    let mut at = 0;
    for _ in 0..100 {
        if at + len > n {
            at = 0;
        }
        S::park(list, at, len, aux, log); // copy only — the list is never mutated
        at += len;
    }
}

// ── Workflow: park a window and place it right back, on a loop ──

/// 20 rounds of `park` then `place` at the same spot (the window slides each
/// round). The buffer fills and drains repeatedly; net identity.
pub struct ParkLoop<S>(PhantomData<S>);
impl<S: Scratchspace> ScratchspaceRun for ParkLoop<S> {
    fn run<T: Ord + Copy, U: ?Sized + SortLogger<T>>(list: &mut [T], log: &mut U) {
        with_scratch::<S, T, U>(list, log, park_loop_body::<S, T, U>);
    }
}
fn park_loop_body<S: Scratchspace, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    list: &mut [T],
    aux: &mut [T],
    log: &mut U,
) {
    let (n, cap) = (list.len(), aux.len());
    if cap == 0 || n == 0 {
        return;
    }
    let len = cap.min((n / 4).max(1));
    let span = n - len + 1; // valid start positions
    let mut at = 0;
    for _ in 0..20 {
        S::park(list, at, len, aux, log);
        S::place(list, at, len, aux, log); // same spot → identity
        at = (at + len) % span;
    }
}

// ── Workflow: relocate a middle block to several targets (near-start/¼/¾) ──

/// Like the rotation kind's target variety: relocate a block to near-start, one
/// -quarter, and three-quarters, returning it each time. Net identity.
pub struct Relocations<S>(PhantomData<S>);
impl<S: Scratchspace> ScratchspaceRun for Relocations<S> {
    fn run<T: Ord + Copy, U: ?Sized + SortLogger<T>>(list: &mut [T], log: &mut U) {
        with_scratch::<S, T, U>(list, log, relocations_body::<S, T, U>);
    }
}
fn relocations_body<S: Scratchspace, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    list: &mut [T],
    aux: &mut [T],
    log: &mut U,
) {
    let n = list.len();
    if n < 8 {
        return;
    }
    let len = (n / 8).max(1);
    let from = n / 2;
    for &to in &[len, n / 4, (3 * n) / 4 - len] {
        let disjoint = (from as isize - to as isize).unsigned_abs() >= len;
        if to + len <= n && from + len <= n && disjoint {
            S::relocate(list, from, to, len, aux, log);
            S::relocate(list, to, from, len, aux, log); // back → identity
        }
    }
}

// ── The nightmare: a scratchspace-backed natural merge sort ──

/// `scratchspace-nightmare`: repeatedly detect two adjacent ascending runs, park
/// the left one, merge them, and move on — pass after pass — until one run spans
/// the array. A real (stable) sort built entirely on the scratchspace: buffered
/// merges when the aux fits, the bufferless rotation merge when it doesn't (so
/// the zero-aux backing pays `O(n log n)` moves for every merge).
pub struct ScratchspaceMergeSort<S>(PhantomData<S>);

impl<S: Scratchspace> ScratchspaceMergeSort<S> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        with_scratch::<S, T, U>(arr, logger, nightmare_body::<S, T, U>);
    }
}

array_vis_bench_traits::direct_sort!(ScratchspaceMergeSort<S>; S: Scratchspace);
// The zero aux backing merges every pass by rotation, `O(n log n)` per pass and
// at most `log n` passes. A backing that buffers does better, so this is the
// bound of the worst member.
impl<S> HasTimeBounds for ScratchspaceMergeSort<S> {
    const WORST: Complexity = Complexity::N_LOG_SQUARED;
    const BEST: Complexity = Complexity::N1; // already sorted → one run, no merges
    const AVERAGE: Complexity = Complexity::N_LOG_SQUARED;
}
impl<S> HasSpace for ScratchspaceMergeSort<S> {
    const SPACE: Complexity = Complexity::N1;
}
impl<S> HasStability for ScratchspaceMergeSort<S> {
    const STABLE: bool = true;
}

fn nightmare_body<S: Scratchspace, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    aux: &mut [T],
    log: &mut U,
) {
    let n = arr.len();
    if n < 2 {
        return;
    }
    loop {
        let mut lo = 0;
        let mut merged = false;
        while lo < n {
            let mid = run_end(arr, lo, log);
            if mid >= n {
                break; // last run has no partner this pass
            }
            let hi = run_end(arr, mid, log);
            merge_runs::<S, T, U>(arr, lo, mid, hi, aux, log);
            merged = true;
            lo = hi;
        }
        if !merged {
            break; // one run left → sorted
        }
    }
}

/// End (exclusive) of the ascending run starting at `start`.
fn run_end<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], start: usize, log: &mut U) -> usize {
    let n = arr.len();
    let mut i = start;
    while i + 1 < n && !log.cmp_lt(arr, i + 1, i) {
        i += 1;
    }
    i + 1
}

/// Stably merge the sorted runs `arr[lo..mid]` and `arr[mid..hi]`, parking the
/// left run when it fits, else with the bufferless rotation merge.
fn merge_runs<S: Scratchspace, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    mid: usize,
    hi: usize,
    aux: &mut [T],
    log: &mut U,
) {
    let a = mid - lo;
    if a <= aux.len() {
        S::park(arr, lo, a, aux, log); // left run → aux
        let (mut i, mut j, mut k) = (0, mid, lo);
        while i < a && j < hi {
            if log.cmp_le_accross(aux, i, arr, j) {
                log.write_accross(aux, i, arr, k); // take left (stable on tie)
                i += 1;
            } else {
                log.write(arr, k, j);
                j += 1;
            }
            k += 1;
        }
        while i < a {
            log.write_accross(aux, i, arr, k);
            i += 1;
            k += 1;
        }
    } else {
        bufferless_merge(arr, lo, mid, hi, log);
    }
}
