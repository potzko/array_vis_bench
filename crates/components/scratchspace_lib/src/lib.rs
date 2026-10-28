//! Auxiliary (scratchspace) backings + the merge that consumes them.
//!
//! The [`Auxiliary`] role (in `array_vis_bench_traits`) is the composition axis
//! behind block-merge sorts (WikiSort / GrailSort): *how much* working buffer a
//! merge is given. This crate provides three backings — [`FullBuffer`] (`O(n)`),
//! [`HalfBuffer`] (`O(n/2)`), [`NoBuffer`] (`O(1)`, in-place) — and
//! [`AuxiliaryMerge`], which turns any `Auxiliary` into a
//! [`RotationMerge`](merge_sort_lib::RotationMerge) so it drops straight into the
//! existing merge pipeline, bench, and visualiser.
//!
//! **Every element move goes through the logger** — `copy_range` / `write` /
//! `write_accross` for the buffered path, rotations for the in-place path, so the
//! array (and the external buffer) animate in the visualiser with no bespoke
//! wiring. The buffer itself is allocated by the pipeline via the logger
//! (`create_aux_arr_t`), exactly like [`Rotation`](array_vis_bench_traits::role::Rotation).

use core::marker::PhantomData;

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::role::{Auxiliary, Scratchspace};
use array_vis_bench_traits::Complexity;
use merge_sort_lib::RotationMerge;
use sort_logger::SortLogger;

pub mod bufferless;
pub mod workflows;
pub use bufferless::bufferless_merge;
pub use workflows::{Exercise, ParkLoop, ParkSweep, Relocations, ScratchspaceMergeSort};

// ── Scratchspace backings (the `Scratchspace` role — its own top-level kind) ──
// They realise the SAME operation set (park/place/relocate/block_swap/roll) via
// the default bodies; they differ only in how much auxiliary they request, which
// selects the strategy the defaults take (fully buffered → chunked → rotation).

/// `O(n)` aux — every relocate fits, so the whole contract runs buffered.
pub struct FullScratch;
impl Scratchspace for FullScratch {
    const NAME: &'static str = "full";
    fn aux_size(n: usize) -> usize {
        n
    }
}
impl HasSpace for FullScratch {
    const SPACE: Complexity = Complexity::N1;
}

/// `O(√n)` aux — the classic block-merge budget: small runs park, larger ones
/// fall back to a rotation.
pub struct SqrtScratch;
impl Scratchspace for SqrtScratch {
    const NAME: &'static str = "sqrt";
    fn aux_size(n: usize) -> usize {
        n.isqrt()
    }
}
impl HasSpace for SqrtScratch {
    const SPACE: Complexity = Complexity::SQRT_N;
}

/// `O(1)` aux — fully in-place: `park` is unavailable, so `relocate`/`roll` run
/// entirely by rotation. The bufferless member of the contract.
pub struct NoScratch;
impl Scratchspace for NoScratch {
    const NAME: &'static str = "none";
    fn aux_size(_n: usize) -> usize {
        0
    }
}
impl HasSpace for NoScratch {
    const SPACE: Complexity = Complexity::CONST;
}

/// `O(n)` buffer: always big enough to hold either run, so merges are the
/// buffered (`O(n)`) copy-merge. Textbook full-copy auxiliary.
pub struct FullBuffer;
impl Auxiliary for FullBuffer {
    const NAME: &'static str = "full";
    const MERGE_WORST: Complexity = Complexity::N1;
    const MERGE_SPACE: Complexity = Complexity::N1;
    fn scratch_size(n: usize) -> usize {
        n
    }
}

/// `O(n/2)` buffer: holds the (smaller-or-equal) left run of a balanced merge,
/// so a balanced merge stays buffered; a skewed one falls back to in-place.
pub struct HalfBuffer;
impl Auxiliary for HalfBuffer {
    const NAME: &'static str = "half";
    const MERGE_WORST: Complexity = Complexity::N1;
    const MERGE_SPACE: Complexity = Complexity::N1; // n/2 is still O(n)
    fn scratch_size(n: usize) -> usize {
        n.div_ceil(2)
    }
}

/// No buffer: every merge is the bufferless rotation merge, `O(1)` space.
/// The bufferless member of the axis.
pub struct NoBuffer;
impl Auxiliary for NoBuffer {
    const NAME: &'static str = "none";
    const MERGE_WORST: Complexity = Complexity::N_SQUARED;
    const MERGE_SPACE: Complexity = Complexity::CONST;
    fn scratch_size(_n: usize) -> usize {
        0
    }
}

/// A [`RotationMerge`] whose working buffer is chosen by the [`Auxiliary`] `A`.
/// The pipeline allocates `A::scratch_size(n)` cells; if that can hold the left
/// run the merge is the logged copy-merge, otherwise it degrades to the logged
/// bufferless rotation merge.
pub struct AuxiliaryMerge<A> {
    _phantom: PhantomData<A>,
}

impl<A: Auxiliary> RotationMerge for AuxiliaryMerge<A> {
    #[inline]
    fn scratch_size(n: usize) -> usize {
        A::scratch_size(n)
    }

    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        mid: usize,
        scratch: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        if mid == 0 || mid >= n {
            return; // one side empty → already merged
        }
        if scratch.len() >= mid {
            half_copy_merge(arr, mid, scratch, logger);
        } else {
            bufferless_merge(arr, 0, mid, n, logger);
        }
    }
}

// Complexity/stability inherited by the emitted `AlgorithmEntry` from the backing.
impl<A: Auxiliary> HasTimeBounds for AuxiliaryMerge<A> {
    const WORST: Complexity = A::MERGE_WORST;
    const BEST: Complexity = Complexity::N1; // one linear pass on already-merged input
    const AVERAGE: Complexity = A::MERGE_WORST;
}
impl<A: Auxiliary> HasSpace for AuxiliaryMerge<A> {
    const SPACE: Complexity = A::MERGE_SPACE;
}
impl<A: Auxiliary> HasStability for AuxiliaryMerge<A> {
    const STABLE: bool = true;
}

/// Copy the left run into the buffer, then merge back — every move logged, so
/// the external buffer visibly fills and drains. Stable (`<=` takes the left
/// run on ties). Requires `scratch.len() >= mid`.
fn half_copy_merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    mid: usize,
    scratch: &mut [T],
    logger: &mut U,
) {
    let n = arr.len();
    logger.copy_range(arr, 0, scratch, 0, mid); // scratch[..mid] = arr[..mid]
    let mut i = 0; // scratch (left run)
    let mut j = mid; // arr (right run)
    let mut k = 0; // write head (k <= j invariantly, so unread right is safe)
    while i < mid && j < n {
        if logger.cmp_le_accross(scratch, i, arr, j) {
            logger.write_accross(scratch, i, arr, k); // arr[k] = scratch[i]
            i += 1;
        } else {
            logger.write(arr, k, j); // arr[k] = arr[j]
            j += 1;
        }
        k += 1;
    }
    while i < mid {
        logger.write_accross(scratch, i, arr, k);
        i += 1;
        k += 1;
    }
    // Remaining arr[j..n] already in place.
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cmp::Ordering;
    use sort_logger::{NoOpLogger, SortLog};

    /// Ordered by `key` only — equal keys are real ties a stable merge must
    /// preserve. `tag` is a globally-unique original-position marker.
    #[derive(Clone, Copy, Debug)]
    struct Item {
        key: u32,
        tag: u32,
    }
    impl PartialEq for Item {
        fn eq(&self, o: &Self) -> bool {
            self.key == o.key
        }
    }
    impl Eq for Item {}
    impl PartialOrd for Item {
        fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
            Some(self.cmp(o))
        }
    }
    impl Ord for Item {
        fn cmp(&self, o: &Self) -> Ordering {
            self.key.cmp(&o.key)
        }
    }

    fn assert_sorted_stable(region: &[Item]) {
        for w in region.windows(2) {
            assert!(w[0].key <= w[1].key, "not sorted: {region:?}");
            if w[0].key == w[1].key {
                assert!(w[0].tag < w[1].tag, "unstable: {region:?}");
            }
        }
    }

    fn multiset(items: &[Item]) -> Vec<(u32, u32)> {
        let mut v: Vec<(u32, u32)> = items.iter().map(|i| (i.key, i.tag)).collect();
        v.sort_unstable();
        v
    }

    /// Merge two sorted halves (keys ascending; left tags precede right tags so a
    /// stable merge keeps left first on ties) via one backing; check the result.
    fn merge_case<A: Auxiliary>(left_keys: &[u32], right_keys: &[u32]) {
        let mid = left_keys.len();
        let mut arr: Vec<Item> = left_keys
            .iter()
            .enumerate()
            .map(|(i, &k)| Item { key: k, tag: i as u32 })
            .chain(
                right_keys
                    .iter()
                    .enumerate()
                    .map(|(i, &k)| Item { key: k, tag: (mid + i) as u32 }),
            )
            .collect();
        let before = multiset(&arr);
        let n = arr.len();
        let mut scratch = vec![Item { key: 0, tag: 0 }; <AuxiliaryMerge<A> as RotationMerge>::scratch_size(n)];
        <AuxiliaryMerge<A> as RotationMerge>::merge(&mut arr, mid, &mut scratch, &mut NoOpLogger);
        assert_sorted_stable(&arr);
        assert_eq!(multiset(&arr), before, "elements lost/duplicated");
    }

    fn merge_battery<A: Auxiliary>() {
        merge_case::<A>(&[], &[1, 2, 3]); // empty left
        merge_case::<A>(&[1, 2, 3], &[]); // empty right
        merge_case::<A>(&[1], &[1]); // equal-key tie
        merge_case::<A>(&[1, 3, 5, 7], &[2, 4, 6, 8]); // interleave
        merge_case::<A>(&[1, 2, 2, 4], &[2, 3, 4, 5]); // heavy duplicates across the seam
        merge_case::<A>(&[5, 6, 7], &[1, 2, 3]); // right entirely below left
    }

    #[test]
    fn full_buffer_merges_stably() {
        merge_battery::<FullBuffer>();
    }
    #[test]
    fn half_buffer_merges_stably() {
        merge_battery::<HalfBuffer>();
    }
    #[test]
    fn no_buffer_merges_stably() {
        merge_battery::<NoBuffer>();
    }

    /// `usize` (the project's element type) cross-checked against a reference merge.
    fn usize_check<A: Auxiliary>() {
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = |m: usize| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            ((state >> 33) as usize) % m
        };
        for &(la, lb) in &[(0usize, 4usize), (4, 0), (8, 8), (17, 15), (1, 1), (64, 64)] {
            let mut left: Vec<usize> = (0..la).map(|_| next(50)).collect();
            let mut right: Vec<usize> = (0..lb).map(|_| next(50)).collect();
            left.sort();
            right.sort();
            let mid = la;
            let mut arr: Vec<usize> = left.iter().chain(&right).copied().collect();
            let mut expected = arr.clone();
            expected.sort();
            let mut scratch =
                vec![0usize; <AuxiliaryMerge<A> as RotationMerge>::scratch_size(arr.len())];
            <AuxiliaryMerge<A> as RotationMerge>::merge(&mut arr, mid, &mut scratch, &mut NoOpLogger);
            assert_eq!(arr, expected, "mismatch at ({la},{lb})");
        }
    }

    #[test]
    fn usize_full() {
        usize_check::<FullBuffer>();
    }
    #[test]
    fn usize_half() {
        usize_check::<HalfBuffer>();
    }
    #[test]
    fn usize_none() {
        usize_check::<NoBuffer>();
    }

    // ── Workflows: each nets to identity for every backing; plus per-operation
    //    contracts and the scratchspace-backed nightmare sort ──
    use array_vis_bench_traits::role::ScratchspaceRun;

    /// A workflow `W` (backing baked in) nets to identity across sizes/shapes.
    fn workflow_is_identity<W: ScratchspaceRun>() {
        for &n in &[0usize, 1, 3, 4, 8, 17, 64, 100, 257] {
            for shape in 0..3 {
                let original: Vec<usize> = match shape {
                    0 => (0..n).collect(),
                    1 => (0..n).rev().collect(),
                    _ => (0..n).map(|i| i % 5).collect(),
                };
                let mut arr = original.clone();
                W::run(&mut arr, &mut NoOpLogger);
                assert_eq!(arr, original, "workflow not identity (n={n}, shape={shape})");
            }
        }
    }

    #[test]
    fn exercise_nets_to_identity() {
        workflow_is_identity::<Exercise<FullScratch>>();
        workflow_is_identity::<Exercise<SqrtScratch>>();
        workflow_is_identity::<Exercise<NoScratch>>();
    }
    #[test]
    fn park_sweep_nets_to_identity() {
        workflow_is_identity::<ParkSweep<FullScratch>>();
        workflow_is_identity::<ParkSweep<SqrtScratch>>();
        workflow_is_identity::<ParkSweep<NoScratch>>();
    }
    #[test]
    fn park_loop_nets_to_identity() {
        workflow_is_identity::<ParkLoop<FullScratch>>();
        workflow_is_identity::<ParkLoop<SqrtScratch>>();
        workflow_is_identity::<ParkLoop<NoScratch>>();
    }
    #[test]
    fn relocations_net_to_identity() {
        workflow_is_identity::<Relocations<FullScratch>>();
        workflow_is_identity::<Relocations<SqrtScratch>>();
        workflow_is_identity::<Relocations<NoScratch>>();
    }

    /// The nightmare sort actually sorts (vs `std`), on every backing.
    fn nightmare_sorts<S: Scratchspace>() {
        let mut state: u64 = 0x00C0_FFEE;
        let mut next = |m: usize| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            ((state >> 33) as usize) % m
        };
        for &n in &[0usize, 1, 2, 8, 33, 128, 257] {
            let data: Vec<usize> = (0..n).map(|_| next(20)).collect();
            let mut arr = data.clone();
            ScratchspaceMergeSort::<S>::sort(&mut arr, &mut NoOpLogger);
            let mut expected = data;
            expected.sort();
            assert_eq!(arr, expected, "nightmare<{}> mismatch at n={n}", S::NAME);
        }
    }
    #[test]
    fn nightmare_full_sorts() {
        nightmare_sorts::<FullScratch>();
    }
    #[test]
    fn nightmare_sqrt_sorts() {
        nightmare_sorts::<SqrtScratch>();
    }
    #[test]
    fn nightmare_none_sorts() {
        nightmare_sorts::<NoScratch>();
    }

    // Per-operation contracts (checked against `FullScratch`, which buffers all).
    #[test]
    fn park_then_place_returns_the_run() {
        let mut arr: Vec<usize> = (0..16).collect();
        let mut aux = vec![0usize; 16];
        FullScratch::park(&mut arr, 4, 5, &mut aux, &mut NoOpLogger);
        // the parked run now lives in aux[0..5]
        assert_eq!(&aux[0..5], &[4, 5, 6, 7, 8]);
        FullScratch::place(&mut arr, 10, 5, &mut aux, &mut NoOpLogger); // drop it at 10
        assert_eq!(&arr[10..15], &[4, 5, 6, 7, 8]);
    }

    #[test]
    fn relocate_moves_a_block_and_conserves() {
        // move [10,11,12] (at idx 2) to the front; both directions.
        let mut arr = vec![0, 1, 10, 11, 12, 5, 6, 7];
        FullScratch::relocate(&mut arr, 2, 0, 3, &mut vec![0usize; 8], &mut NoOpLogger);
        assert_eq!(arr, vec![10, 11, 12, 0, 1, 5, 6, 7]);
        // and back
        FullScratch::relocate(&mut arr, 0, 2, 3, &mut vec![0usize; 8], &mut NoOpLogger);
        assert_eq!(arr, vec![0, 1, 10, 11, 12, 5, 6, 7]);
    }

    #[test]
    fn relocate_without_aux_falls_back_to_rotation() {
        // NoScratch has zero aux → relocate must still move the block (via roll).
        let mut arr = vec![0, 1, 10, 11, 12, 5, 6, 7];
        NoScratch::relocate(&mut arr, 2, 0, 3, &mut [], &mut NoOpLogger);
        assert_eq!(arr, vec![10, 11, 12, 0, 1, 5, 6, 7]);
    }

    #[test]
    fn roll_left_rotates_a_range() {
        let mut arr = vec![0, 1, 2, 3, 4, 5];
        FullScratch::roll(&mut arr, 1, 5, 2, &mut [], &mut NoOpLogger); // rotate [1..5) left by 2
        assert_eq!(arr, vec![0, 3, 4, 1, 2, 5]);
    }

    // Counts the logged moves and comparisons, so a test can read a growth rate.
    #[derive(Default)]
    struct Tally {
        moves: usize,
        cmps: usize,
    }
    impl SortLogger<usize> for Tally {
        fn log(&mut self, event: SortLog<usize>) {
            match event {
                SortLog::Swap { .. }
                | SortLog::WriteInArr { .. }
                | SortLog::WriteData { .. }
                | SortLog::WriteDataU { .. } => self.moves += 1,
                SortLog::CmpInArr { .. }
                | SortLog::CmpData { .. }
                | SortLog::CmpDataU { .. }
                | SortLog::CmpAcrossArrs { .. } => self.cmps += 1,
                _ => {}
            }
        }
    }

    // Two sorted runs of `n / 2` each, interleaved evenly (`even`) or shuffled by a
    // fixed generator.
    fn two_runs(n: usize, even: bool) -> Vec<usize> {
        let mut state: u64 = 0x5EED_0001;
        let mut next = |m: usize| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((state >> 33) as usize) % m
        };
        let mut left: Vec<usize> = (0..n / 2).map(|i| if even { 2 * i } else { next(4 * n) }).collect();
        let mut right: Vec<usize> = (0..n / 2).map(|i| if even { 2 * i + 1 } else { next(4 * n) }).collect();
        left.sort();
        right.sort();
        left.into_iter().chain(right).collect()
    }

    fn no_buffer_cost(n: usize, even: bool) -> Tally {
        let mut arr = two_runs(n, even);
        let mut expected = arr.clone();
        expected.sort();
        let mut tally = Tally::default();
        <AuxiliaryMerge<NoBuffer> as RotationMerge>::merge(&mut arr, n / 2, &mut [], &mut tally);
        assert_eq!(arr, expected);
        tally
    }

    // Growing the input 16 times grows an n log n cost about 24 times and a
    // quadratic one 256 times; the line sits between with room for the constant.
    #[test]
    fn no_buffer_merge_grows_as_n_log_n() {
        for even in [true, false] {
            let (small, large) = (no_buffer_cost(256, even), no_buffer_cost(4096, even));
            assert!(large.moves < 40 * small.moves, "moves {} -> {}", small.moves, large.moves);
            assert!(large.cmps < 40 * small.cmps, "comparisons {} -> {}", small.cmps, large.cmps);
        }
    }
}
