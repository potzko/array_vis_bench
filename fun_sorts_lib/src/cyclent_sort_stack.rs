//! Cyclent sort — iterative quicksort (v3).
//!
//! The limit of the cyclent / stack chain: instead of tracking a running
//! `i` plus a stack of lower bounds, store the actual sub-problems as
//! `(lo, hi)` frames on a stack. Pop a frame, partition its slice with
//! the last element as pivot, push the two halves. Continue until the
//! stack is empty.
//!
//! Behaviourally this is plain iterative quicksort; the cyclent
//! framing's right-to-left `i` pointer has dissolved into "always work
//! on whatever frame is on top". The stack-optimized variant
//! (`CyclentSortStackOptimized`) is the same algorithm dressed up to
//! preserve the `i` pointer — useful for visualisation, but equivalent
//! work.

use std::marker::PhantomData;

use array_vis_bench_traits::PartitionScheme;
use sort_logger::SortLogger;

pub struct CyclentSortStack<P: PartitionScheme>(PhantomData<P>);

impl<P: PartitionScheme> CyclentSortStack<P> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        let mut stack: Vec<(usize, usize)> = Vec::new();
        if n >= 2 {
            stack.push((0, n));
        }
        while let Some((lo, hi)) = stack.pop() {
            if hi - lo < 2 {
                continue;
            }
            let slice_len = hi - lo;
            let (l, r) = P::partition(&mut arr[lo..hi], logger, slice_len - 1);
            let pivot_end = lo + l;
            let right_start = lo + r;
            // Push right half first so the left half is processed next
            // (LIFO) — keeps the recursion shape similar to the
            // cyclent-framed variants.
            if right_start < hi {
                stack.push((right_start, hi));
            }
            if lo < pivot_end {
                stack.push((lo, pivot_end));
            }
        }
    }
}

// MovingPivot is excluded — see cyclent_sort.rs for rationale.
