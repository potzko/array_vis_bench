//! What a block partition does once its blocks are made: `by_blocks` trims the
//! ends that are in place, hands the region to a function that makes whole
//! blocks of one type each, `order_blocks` puts the blocks in order, front ones
//! first, remembering which block was which by writing its rank into the
//! elements, and one rotation places the loose elements. `EncodedPartition` and
//! `MaskedPartition` differ in how they make the blocks and in nothing else.

use array_vis_bench_traits::role::keys::rotate_region;
use array_vis_bench_traits::role::Rotation;
use sort_logger::SortLogger;

use crate::partition::buffered;

pub(crate) fn ceil_log2(n: usize) -> usize { (usize::BITS - (n - 1).leading_zeros()) as usize }

// `front` says whether the element at an index is one of those to bring to the
// front, by one logged comparison against the pivot, and `b` is the block length.
// The front elements the region starts with and the back elements it ends with
// are in place, and are read once and left, so a pass with nothing to move costs
// its comparisons alone; what is between them, which starts with a back element
// and ends with a front one, is partitioned.
//
// `gather` is handed that region, `arr[lo..hi]` and the scratch, and leaves
// whole blocks of one type each in `arr[lo..end]`, each type's in order, then
// `loose` front elements, fewer than a block, then the rest of the back
// elements, fewer than a block, in order, and returns `(end, loose)`.
pub(crate) fn by_blocks<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool, G>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    b: usize,
    front: &F,
    scratch: &mut [T],
    log: &mut U,
    gather: G,
) -> usize
where
    G: FnOnce(&mut [T], usize, usize, &mut [T], &mut U) -> (usize, usize),
{
    let Some(start) = (lo..hi).find(|&i| !front(log, arr, i)) else { return hi };
    let Some(last) = (start + 1..hi).rev().find(|&i| front(log, arr, i)) else { return start };
    let (lo, hi) = (start, last + 1);
    if hi - lo <= scratch.len() { return buffered(arr, lo, hi, front, scratch, log); }
    let (end, loose) = gather(arr, lo, hi, scratch, log);
    let fronts = order_blocks(arr, lo, (end - lo) / b, b, front, log);
    // The loose front elements rotate in front of the back blocks.
    rotate_region::<R, T, U>(arr, lo + fronts * b, end, end + loose, scratch, log);
    lo + fronts * b + loose
}

// Put the `m` blocks of `b` cells from `lo` in order, the front blocks first,
// each type's in the order it had, and return how many are front. A block's
// type is the type of its last cell, which nothing below touches.
//
// What a block swap cannot do is remember which block was which, and the
// blocks of the minority type are the ones that get scrambled, so they carry
// their rank with them. A minority block and a majority block that are the
// `j`th of their type make a pair, and the bits of `j` are written into the
// first cells of the two by swapping a cell of one with the same cell of the
// other for each bit that is set: the pivot tells the two elements apart, so
// the minority block then holds a majority element in exactly the cells of the
// bits of its rank, and no element has to be distinct from another. The
// majority blocks sweep to their end of the row by block swaps, in order, past
// a gap of minority blocks that comes out scrambled, the minority blocks read
// their ranks off the cells and swap into place by cycle leader, each swap
// settling one block, and the swaps of the code are undone, which needs no
// comparison since the block at `j` has the rank `j`.
fn order_blocks<T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    m: usize,
    b: usize,
    front: &F,
    log: &mut U,
) -> usize {
    let cell = |block: usize, t: usize| lo + block * b + t;
    let is_front = |log: &mut U, arr: &[T], block: usize| front(log, arr, cell(block, b - 1));
    let fronts = (0..m).filter(|&block| is_front(log, arr, block)).count();
    let minors = fronts.min(m - fronts);
    if minors == 0 { return fronts; }
    let minor_front = fronts < m - fronts;
    let bits = ceil_log2(minors);
    debug_assert!(bits < b, "{minors} minority blocks need {bits} code cells in blocks of {b}");
    let set = |j: usize| (0..bits).filter(move |t| (j >> t) & 1 == 1);
    // Pair up and write the ranks.
    if bits > 0 {
        let next = |log: &mut U, arr: &[T], from: usize, wanted: bool| (from..m).find(|&block| is_front(log, arr, block) == wanted).unwrap();
        let (mut f, mut k) = (0, 0);
        for j in 0..minors {
            (f, k) = (next(log, arr, f, true), next(log, arr, k, false));
            set(j).for_each(|t| log.swap(arr, cell(f, t), cell(k, t)));
            (f, k) = (f + 1, k + 1);
        }
    }
    // Sweep the majority blocks to one end, from the left when they are front
    // blocks and from the right when they are back blocks.
    let at = |s: usize| if minor_front { m - 1 - s } else { s };
    let mut placed = 0;
    for s in 0..m {
        if is_front(log, arr, at(s)) != minor_front {
            if placed != s { log.block_swap(arr, cell(at(placed), 0), cell(at(s), 0), b); }
            placed += 1;
        }
    }
    // The minority blocks are at the other end, in no order.
    let base = if minor_front { 0 } else { fronts };
    for p in base..base + minors {
        loop {
            let rank = (0..bits).filter(|&t| front(log, arr, cell(p, t)) != minor_front).fold(0, |rank, t| rank | 1 << t);
            if rank == p - base { break; }
            log.block_swap(arr, cell(p, 0), cell(base + rank, 0), b);
        }
    }
    for j in 0..minors { set(j).for_each(|t| log.swap(arr, cell(j, t), cell(fronts + j, t))); }
    fronts
}
