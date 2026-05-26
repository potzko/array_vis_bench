//! Cyclent sort — single-bound optimized variant (v1).
//!
//! Like the plain right-to-left cyclent sort, but remembers the previous
//! partition's `right_start` as a *lower* bound — within a single
//! outer-iteration's max-search. After partition returns `(_, r)` with
//! `r < slice_len`, we know everything in `arr[bound..bound+r]` is ≤ the
//! old pivot and therefore < the new (larger) pivot — the next inner
//! partition can skip them.
//!
//! Single-slot: the bound resets to `0` between outer iterations because
//! we don't remember earlier bounds. The stack-optimized variant lifts
//! that restriction by keeping all bounds; this one is the minimal
//! optimization over the plain variant.

use std::marker::PhantomData;
use std::ops::Range;

use array_vis_bench_traits::{PartitionScheme, PartitionVisitor};
use sort_logger::SortLogger;

pub struct CyclentSortOpt<P: PartitionScheme>(PhantomData<P>);

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

impl<P: PartitionScheme> CyclentSortOpt<P> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        for i in (0..n).rev() {
            let mut bound = 0;
            loop {
                let slice_len = i + 1 - bound;
                if slice_len < 2 {
                    break;
                }
                let mut v = RightStart { right_start: slice_len, n: 0 };
                P::partition(&mut arr[bound..=i], logger, &[slice_len - 1], &mut v);
                if v.right_start == slice_len {
                    break;
                }
                bound += v.right_start;
            }
        }
    }
}

// MovingPivot is excluded — see cyclent_sort.rs for rationale.
