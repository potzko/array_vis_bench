//! Cyclent sort — plain variant (v0).
//!
//! Runs **right to left**: for each `i` descending from `n-1`, repeatedly
//! partition `arr[0..=i]` with `arr[i]` as the pivot until the pivot's
//! range reaches the slice's end (no elements remain `>` it). Each
//! partition that finds something larger surfaces it at position `i`,
//! shrinking the search. When the partition's `right_start` equals the
//! slice length, `arr[i]` is the max of the slice — advance `i`.
//!
//! Right-to-left flip is what lets us reuse the quick-sort partitions
//! verbatim. Their `≤` semantics (equal elements go *left*) means the
//! pivot's range can extend through to the slice's end on duplicate-heavy
//! input, which is exactly the convergence condition.
//!
//! No memory across iterations; every inner partition scans `arr[0..=i]`
//! from scratch. The optimized variant narrows the lower bound.

use std::marker::PhantomData;
use std::ops::Range;

use array_vis_bench_traits::{with_partition_scratch, PartitionScheme, PartitionVisitor};
use sort_logger::SortLogger;

pub struct CyclentSort<P: PartitionScheme>(PhantomData<P>);

/// Captures `right_start` from a single-pivot partition: the start of
/// the second unsorted range, or `len` if the partition placed
/// everything to the left of the pivot.
struct RightStart { right_start: usize, n: u8 }
impl PartitionVisitor for RightStart {
    #[inline(always)]
    fn unsorted(&mut self, r: Range<usize>) {
        if self.n == 1 {
            self.right_start = r.start;
        }
        self.n += 1;
    }
}

impl<P: PartitionScheme> CyclentSort<P> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        with_partition_scratch::<P, T, U, _>(logger, |logger, scratch| {
            for i in (0..n).rev() {
                loop {
                    let slice_len = i + 1;
                    if slice_len < 2 {
                        break;
                    }
                    let mut v = RightStart { right_start: slice_len, n: 0 };
                    P::partition(&mut arr[..slice_len], logger, &[slice_len - 1], scratch, &mut v);
                    if v.right_start == slice_len {
                        break;
                    }
                }
            }
        });
    }
}

// MovingPivot is excluded — it returns `(high, high)` with `high < len` always,
// so the convergence check `r == slice_len` would never fire (infinite loop).
