//! `Stash`, where the left run of a local merge waits. A merge writes its output
//! over the left run's own cells, so the run is first parked out of the way and
//! its cells become scratch. `Stash::Internal` parks into a buffer region of the
//! array by swapping, so the buffer's values end up in the array's scratch cells
//! and come back when the merge is done. `Stash::External` copies into an aux
//! array, and its leftovers are stale copies that nobody reads. `Stash::None` has
//! nowhere to park, the run stays in place and the merge is the rotation merge,
//! over the rotation the sort was given.

use array_vis_bench_traits::role::Rotation;
use scratchspace_lib::bufferless_merge;
use sort_logger::SortLogger;

pub enum Stash<'a, T> {
    None,
    /// `cap` cells of the array from `at`.
    Internal { at: usize, cap: usize },
    External(&'a mut [T]),
}

impl<T: Ord + Copy> Stash<'_, T> {
    /// The longest run the stash can park.
    pub fn capacity(&self) -> usize {
        match self {
            Stash::None => 0,
            Stash::Internal { cap, .. } => *cap,
            Stash::External(ext) => ext.len(),
        }
    }

    /// Park `arr[from..from + len]`. The cells are scratch from here on.
    pub fn park<U: ?Sized + SortLogger<T>>(&mut self, arr: &mut [T], from: usize, len: usize, log: &mut U) {
        match self {
            Stash::None => {}
            Stash::Internal { at, .. } => log.block_swap(arr, from, *at, len),
            Stash::External(ext) => log.copy_range(arr, from, ext, 0, len),
        }
    }

    /// Merge the parked run of `a_len` elements, whose cells start at `a_start`,
    /// with the `b_len` elements behind those cells. The output fills `a_len +
    /// b_len` cells from `a_start`, an equal key of the run ahead of one of B.
    /// `scratch` is `R`'s, for the rotation merge of `Stash::None`.
    pub fn merge<R: Rotation, U: ?Sized + SortLogger<T>>(
        &mut self,
        arr: &mut [T],
        a_start: usize,
        a_len: usize,
        b_len: usize,
        scratch: &mut [T],
        log: &mut U,
    ) {
        match self {
            Stash::None => bufferless_merge::<R, T, U>(arr, a_start, a_start + a_len, a_start + a_len + b_len, scratch, log),
            Stash::Internal { at, .. } => swap_merge(arr, a_start, a_len, b_len, *at, log),
            Stash::External(ext) => copy_merge(arr, ext, a_start, a_len, b_len, log),
        }
    }

    /// Move the `rem` elements at `arr[from..from + rem]` behind the `len` cells
    /// of a block that follow them, `slots` being the first of those. A parked
    /// block's cells are scratch, so a block swap does it, and a block still in
    /// place is rotated past them by `R`.
    pub fn pass_behind<R: Rotation, U: ?Sized + SortLogger<T>>(
        &mut self,
        arr: &mut [T],
        from: usize,
        slots: usize,
        len: usize,
        rem: usize,
        scratch: &mut [T],
        log: &mut U,
    ) {
        match self {
            Stash::None => R::rotate(&mut arr[from..slots + len], rem, scratch, log),
            _ => log.block_swap(arr, from, slots + len - rem, rem),
        }
    }
}

// The parked run is in the array's buffer cells `at..`, the cells of its slots
// hold scratch. Every output step swaps the output cell with the source, so the
// scratch drifts into the cells the sources leave and the gap between the output
// head and the B head is exactly the run still parked. When B runs out the rest
// of the run is swapped in, and all the scratch is back in the buffer.
fn swap_merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    a_start: usize,
    a_len: usize,
    b_len: usize,
    at: usize,
    log: &mut U,
) {
    let (mut i, mut j, mut out) = (0, a_start + a_len, a_start);
    let end = j + b_len;
    while i < a_len && j < end {
        if log.cmp_lt(arr, j, at + i) {
            log.swap(arr, out, j);
            j += 1;
        } else {
            log.swap(arr, out, at + i);
            i += 1;
        }
        out += 1;
    }
    log.block_swap(arr, at + i, out, a_len - i);
}

// The run was copied out, so its cells are free to overwrite. The write head
// never passes the B head, because it trails by the run still waiting.
fn copy_merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    ext: &[T],
    a_start: usize,
    a_len: usize,
    b_len: usize,
    log: &mut U,
) {
    let (mut i, mut j, mut out) = (0, a_start + a_len, a_start);
    let end = j + b_len;
    while i < a_len && j < end {
        if log.cmp_le_accross(ext, i, arr, j) {
            log.write_accross(ext, i, arr, out);
            i += 1;
        } else {
            log.write(arr, out, j);
            j += 1;
        }
        out += 1;
    }
    log.copy_range(ext, i, arr, out, a_len - i);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{junk, merged, pairs, sorted_run, Item, Rng};
    use rotation_reversal::ReversalRotation;
    use sort_logger::NoOpLogger;

    #[test]
    fn every_stash_merges_stably() {
        let mut rng = Rng(0xB10C);
        for round in 0..600 {
            let (a_len, b_len) = (rng.next(14), rng.next(14));
            let values = 1 + rng.next(8);
            let a = sorted_run(&mut rng, a_len, values, 0);
            let b = sorted_run(&mut rng, b_len, values, 100);
            let want = merged(&a, &b);
            let cap = a_len + rng.next(4);

            // internal: [buffer | A | B], A parked by swapping with the buffer
            let mut arr: Vec<Item> = junk(cap).into_iter().chain(a.iter().copied()).chain(b.iter().copied()).collect();
            let mut stash = Stash::Internal { at: 0, cap };
            stash.park(&mut arr, cap, a_len, &mut NoOpLogger);
            stash.merge::<ReversalRotation, _>(&mut arr, cap, a_len, b_len, &mut [], &mut NoOpLogger);
            assert_eq!(pairs(&arr[cap..]), want, "internal, round {round}");
            let mut buffer: Vec<u32> = arr[..cap].iter().map(|x| x.tag).collect();
            buffer.sort();
            assert_eq!(buffer, (5000..5000 + cap as u32).collect::<Vec<_>>(), "the buffer lost a value, round {round}");

            // external: A copied out
            let mut arr: Vec<Item> = a.iter().chain(&b).copied().collect();
            let mut ext = junk(cap);
            let mut stash = Stash::External(&mut ext);
            stash.park(&mut arr, 0, a_len, &mut NoOpLogger);
            stash.merge::<ReversalRotation, _>(&mut arr, 0, a_len, b_len, &mut [], &mut NoOpLogger);
            assert_eq!(pairs(&arr), want, "external, round {round}");

            // none: A stays in place
            let mut arr: Vec<Item> = a.iter().chain(&b).copied().collect();
            let mut stash = Stash::None;
            stash.park(&mut arr, 0, a_len, &mut NoOpLogger);
            stash.merge::<ReversalRotation, _>(&mut arr, 0, a_len, b_len, &mut [], &mut NoOpLogger);
            assert_eq!(pairs(&arr), want, "none, round {round}");
        }
    }

    #[test]
    fn capacity_is_what_each_shape_can_park() {
        let mut ext = [Item { key: 0, tag: 0 }; 5];
        assert_eq!(Stash::<Item>::None.capacity(), 0);
        assert_eq!(Stash::<Item>::Internal { at: 3, cap: 9 }.capacity(), 9);
        assert_eq!(Stash::External(&mut ext).capacity(), 5);
    }

    // Cells 0..9 hold data, 3 and 4 the elements to pass and 5..9 the block, and
    // the 4 cells behind them are the buffer.
    #[test]
    fn a_block_passes_behind_the_elements_in_front_of_it() {
        let (prefix, rem, len) = (3usize, 2usize, 4usize);
        let tags = |arr: &[Item]| arr.iter().map(|x| x.tag).collect::<Vec<_>>();
        for stash in [Stash::Internal { at: 9, cap: 4 }, Stash::None] {
            let parked = !matches!(stash, Stash::None);
            let mut stash = stash;
            let mut arr: Vec<Item> = (0..9).map(|t| Item { key: t, tag: t }).collect();
            arr.extend(junk(4));
            let before = tags(&arr);
            stash.park(&mut arr, prefix + rem, len, &mut NoOpLogger);
            stash.pass_behind::<ReversalRotation, _>(&mut arr, prefix, prefix + rem, len, rem, &mut [], &mut NoOpLogger);
            let after = tags(&arr);
            assert_eq!(&after[prefix + len..prefix + len + rem], &before[prefix..prefix + rem], "the elements are not behind");
            if !parked {
                assert_eq!(&after[prefix..prefix + len], &before[prefix + rem..prefix + rem + len], "the block moved");
            }
            let (mut a, mut b) = (after.clone(), before.clone());
            a.sort();
            b.sort();
            assert_eq!(a, b, "elements lost");
        }
    }
}
