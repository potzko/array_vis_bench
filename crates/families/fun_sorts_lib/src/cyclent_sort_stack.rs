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
use std::ops::Range;

use array_vis_bench_traits::{with_partition_scratch, PartitionScheme, PartitionVisitor};
use sort_logger::SortLogger;

pub struct CyclentSortStack<P: PartitionScheme>(PhantomData<P>);

struct Bounds { left_end: usize, right_start: usize, n: u8 }
impl PartitionVisitor for Bounds {
    #[inline(always)]
    fn unsorted(&mut self, r: Range<usize>) {
        if self.n == 0 {
            self.left_end = r.end;
        } else if self.n == 1 {
            self.right_start = r.start;
        }
        self.n += 1;
    }
}

impl<P: PartitionScheme> CyclentSortStack<P> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        with_partition_scratch::<P, T, U, _>(logger, |logger, scratch| {
            let mut stack: Vec<(usize, usize)> = Vec::new();
            if n >= 2 {
                stack.push((0, n));
            }
            while let Some((lo, hi)) = stack.pop() {
                if hi - lo < 2 {
                    continue;
                }
                let slice_len = hi - lo;
                let mut v = Bounds { left_end: 0, right_start: slice_len, n: 0 };
                P::partition(&mut arr[lo..hi], logger, &[slice_len - 1], scratch, &mut v);
                let pivot_end = lo + v.left_end;
                let right_start = lo + v.right_start;
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
        });
    }
}

// MovingPivot is excluded — see cyclent_sort.rs for rationale.
