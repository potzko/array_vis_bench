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

use crate::sorts::quick_sorts::partitions::PartitionScheme;
use crate::traits::log_traits::SortLogger;

pub struct CyclentSort<P: PartitionScheme>(PhantomData<P>);

impl<P: PartitionScheme> CyclentSort<P> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        for i in (0..n).rev() {
            loop {
                let slice_len = i + 1;
                if slice_len < 2 {
                    break;
                }
                let (_l, r) = P::partition(&mut arr[..slice_len], logger, slice_len - 1);
                if r == slice_len {
                    break;
                }
            }
        }
    }
}

// MovingPivot is excluded — it returns `(high, high)` with `high < len` always,
// so the convergence check `r == slice_len` would never fire (infinite loop).
combo_codegen::family!(
    type = CyclentSort<{P}>,
    uses = [
        "crate::sorts::quick_sorts::partitions::{Block, Hoare, Lomuto, ThreeWay}",
        "super::cyclent_sort::CyclentSort",
    ],
    P: inline [
        ("Lomuto", "lomuto"),
        ("Hoare", "hoare"),
        ("ThreeWay", "three-way"),
        ("Block", "block"),
    ],
    name = "cyclent sort",
    big_o = "O(N^3?)",
    stable = false,
    direct_sort = true,
    path = ["fun sorts", "cyclent sort", "{P}"],
);
