//! `roll_merge`, the block merge: merge two adjacent sorted runs by rolling the
//! blocks of the left run through the right run, so no element of either run is
//! moved further than a block at a time. Every move and every comparison goes
//! through the logger, and the keys that tag the blocks are the `Keys` axis.

use array_vis_bench_traits::role::keys::first_not_below;
use array_vis_bench_traits::role::{Keys, Rotation};
use rotation_reversal::ReversalRotation;
use sort_logger::SortLogger;

use crate::stash::Stash;

// Why it merges. Cut the left run A into a short front A0 and the full blocks
// A1..Ak of `s` elements, with heads h1 <= h2 <= ..., and let c_j be the number of
// elements of B below h_j. Then the merged run is
//     merge(A0, B[..c_1]) ++ merge(A_1, B[c_1..c_2]) ++ ... ++ merge(A_k, B[c_k..])
// since everything in a piece is at least its head and below the next head, and
// an element of B equal to a head follows it. Each piece is a local merge of one
// block with the B elements that land among it, so the whole merge is `O(la + lb)`.
//
// Rolling puts each block beside its B elements. The window is the blocks not yet
// merged, and a B block is rolled by swapping it with the first block of the
// window: the B block jumps to the front of the window, the first block goes to
// the back of it, and `s` swaps moved a whole B block past `k * s` elements. The
// B elements just rolled are `last_b`. When the lowest block's head is not above
// the last element of `last_b`, some of those elements belong after that block, so
// the block is dropped there: the elements of `last_b` below its head close the
// piece before it, and the rest are passed behind it, which is where the next
// piece starts. The piece before it is merged only now, because only now are all
// its B elements known.
//
// Rolling scrambles the blocks. Each carries a tag in its second element, a key
// swapped in from the key region, so the lowest tag in the window is always the
// next block in A's order (heads can repeat). Blocks drop in that order, so the
// tag released at the d'th drop belongs to key slot d and the region is whole
// again at the end. The tag takes the place of the second element, which is why
// a block holds at least two.
//
// The piece before the dropped block is parked in the stash while it waits, so
// its cells are scratch: passing the rest of `last_b` behind the block is then
// a block swap with scratch instead of a rotation.

/// Merge `arr[lo..mid]` and `arr[mid..hi]`, both sorted, stably, rolling blocks
/// of `s >= 2` elements. `arr[keys_at..]` holds one key for each full block of the
/// left run, `la / s`, sorted and distinct, outside `lo..hi`. The keys are as they
/// were when it returns, and so is the stash's buffer up to order.
pub fn roll_merge<K: Keys, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    mid: usize,
    hi: usize,
    s: usize,
    keys_at: usize,
    stash: &mut Stash<T>,
    log: &mut U,
) {
    let (la, lb) = (mid - lo, hi - mid);
    let (front, blocks) = (la % s, la / s);
    assert!(s >= 2 && blocks >= 1 && lb >= 1, "roll_merge needs a block of two, a full block and a right run");
    for j in 0..blocks { K::tag_block(arr, keys_at + j, lo + front + j * s, log); }

    // The cells of the piece waiting to be merged, its elements parked.
    let (mut pending, mut pending_len) = (lo, front);
    stash.park(arr, lo, front, log);
    // The window is `count` blocks from `window`, the B elements not yet rolled
    // follow it, and `last_b..window` are the B elements rolled last.
    let (mut window, mut count) = (lo + front, blocks);
    let (mut last_b, mut lowest, mut dropped) = (window, window, 0);
    loop {
        let unrolled = hi - (window + count * s);
        if unrolled == 0 || (last_b < window && !log.cmp_lt(arr, window - 1, lowest)) {
            if lowest != window { log.block_swap(arr, window, lowest, s); }
            K::tag_block(arr, keys_at + dropped, window, log);
            dropped += 1;
            let split = first_not_below(arr, last_b, window, window, log);
            stash.merge(arr, pending, pending_len, split - (pending + pending_len), log);
            stash.park(arr, window, s, log);
            stash.pass_behind(arr, split, window, s, window - split, log);
            (pending, pending_len, last_b) = (split, s, split + s);
            window += s;
            count -= 1;
            if count == 0 { break; }
            lowest = window + K::find_block(arr, window, s, count, log) * s;
        } else if unrolled < s {
            // The last B block is short, so a block swap cannot roll it: carry it
            // in front of the window by a rotation instead.
            ReversalRotation::rotate(&mut arr[window..window + count * s + unrolled], count * s, &mut [], log);
            (last_b, window, lowest) = (window, window + unrolled, lowest + unrolled);
        } else {
            log.block_swap(arr, window, window + count * s, s);
            if lowest == window { lowest = window + count * s; }
            (last_b, window) = (window, window + s);
        }
    }
    stash.merge(arr, pending, pending_len, hi - (pending + pending_len), log);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{junk, merged, pairs, sorted_run, Item, Rng};
    use scratchspace_lib::{InArrayKeys, NoKeys};
    use sort_logger::NoOpLogger;

    // Which stash a case runs under.
    #[derive(Clone, Copy, Debug)]
    enum Shape {
        Internal,
        External,
        Nowhere,
    }

    // Lay out [keys | buffer | A | B], roll merge, and check the merged run, the
    // key region (exactly as it was) and the buffer (the same values).
    fn case(shape: Shape, rng: &mut Rng, la: usize, lb: usize, s: usize, values: usize) {
        let blocks = la / s;
        let keys: Vec<Item> = (0..blocks + rng.next(3)).map(|i| Item { key: 10_000 + i as u32, tag: 9000 + i as u32 }).collect();
        let cap = s + rng.next(3);
        let buffer = match shape {
            Shape::Internal => junk(cap),
            _ => Vec::new(),
        };
        let a = sorted_run(rng, la, values, 0);
        let b = sorted_run(rng, lb, values, 100);
        let mut arr: Vec<Item> = keys.iter().chain(&buffer).chain(&a).chain(&b).copied().collect();
        let lo = keys.len() + buffer.len();
        let mut ext = junk(cap);
        let mut stash = match shape {
            Shape::Internal => Stash::Internal { at: keys.len(), cap },
            Shape::External => Stash::External(&mut ext),
            Shape::Nowhere => Stash::None,
        };
        roll_merge::<InArrayKeys, _, _>(&mut arr, lo, lo + la, lo + la + lb, s, 0, &mut stash, &mut NoOpLogger);
        let label = format!("{shape:?} la={la} lb={lb} s={s} values={values}");
        assert_eq!(pairs(&arr[lo..]), merged(&a, &b), "{label}");
        assert_eq!(pairs(&arr[..keys.len()]), pairs(&keys), "keys not restored, {label}");
        let mut got: Vec<u32> = arr[keys.len()..lo].iter().map(|x| x.tag).collect();
        got.sort();
        assert_eq!(got, buffer.iter().map(|x| x.tag).collect::<Vec<_>>(), "buffer values changed, {label}");
    }

    #[test]
    fn rolling_merges_stably_under_every_stash() {
        let mut rng = Rng(0xFACADE);
        for round in 0..3000 {
            let s = 2 + rng.next(6);
            let la = s + rng.next(45);
            let lb = 1 + rng.next(45);
            let values = [1, 2, 3, 6, 1000][rng.next(5)];
            for shape in [Shape::Internal, Shape::External, Shape::Nowhere] {
                case(shape, &mut rng, la, lb, s, values);
            }
            let _ = round;
        }
    }

    #[test]
    fn rolling_handles_the_corner_shapes() {
        let mut rng = Rng(7);
        for shape in [Shape::Internal, Shape::External, Shape::Nowhere] {
            // exactly one block, a right run of one, a right run shorter than a block,
            // a run that is a whole number of blocks on either side
            for (la, lb, s) in [(2, 1, 2), (4, 1, 4), (6, 5, 3), (6, 6, 3), (9, 2, 3), (12, 12, 4), (2, 40, 2), (40, 1, 5)] {
                for values in [1, 2, 1000] { case(shape, &mut rng, la, lb, s, values); }
            }
        }
    }

    #[test]
    fn rolling_without_keys_is_the_front_of_the_window() {
        // NoKeys names block 0 and tags nothing, which is right for a merge that
        // never moves a block: one block, so there is nothing to scramble.
        let mut rng = Rng(11);
        let (la, lb, s) = (4usize, 9usize, 4usize);
        let a = sorted_run(&mut rng, la, 5, 0);
        let b = sorted_run(&mut rng, lb, 5, 100);
        let mut arr: Vec<Item> = a.iter().chain(&b).copied().collect();
        roll_merge::<NoKeys, _, _>(&mut arr, 0, la, la + lb, s, 0, &mut Stash::None, &mut NoOpLogger);
        assert_eq!(pairs(&arr), merged(&a, &b));
    }
}
