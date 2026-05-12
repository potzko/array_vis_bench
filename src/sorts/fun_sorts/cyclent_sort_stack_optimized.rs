//! Cyclent sort — stack-optimized variant (v1).
//!
//! Keeps a stack of the absolute indexes at which previous partitions
//! placed their pivots. The invariant: between the top-of-stack value
//! and `i`, the largest element of *that subrange* is at `i`. So each
//! inner partition operates on `arr[top_of_stack..=i]` — the rest of the
//! array, below the bottom-most still-valid bound, is already known to
//! be smaller than everything above it.
//!
//! Bounds persist across outer iterations: as `i` decreases, any bound
//! that's no longer below `i` gets popped (it can't be a useful lower
//! bound for the new search). The right-to-left frame plus a stack of
//! "I've already placed something this large to the right of this index"
//! is exactly what makes this an improvement on the plain variant — no
//! information gets thrown away between iterations.
//!
//! For the equivalent algorithm framed as iterative quicksort (popping
//! (lo, hi) frames rather than tracking `i` plus a bound stack), see
//! `CyclentSortStack`.

use std::marker::PhantomData;

use crate::sorts::quick_sorts::partitions::PartitionScheme;
use crate::traits::log_traits::SortLogger;

pub struct CyclentSortStackOptimized<P: PartitionScheme>(PhantomData<P>);

impl<P: PartitionScheme> CyclentSortStackOptimized<P> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        let mut stack: Vec<usize> = Vec::new();
        for i in (0..n).rev() {
            loop {
                // Drop bounds that no longer sit strictly below `i` —
                // their subrange `[bound..=i]` is empty / single-element.
                while let Some(&top) = stack.last() {
                    if top >= i {
                        stack.pop();
                    } else {
                        break;
                    }
                }
                let bound = stack.last().copied().unwrap_or(0);
                let slice_len = i + 1 - bound;
                if slice_len < 2 {
                    break;
                }
                let (_l, r) = P::partition(&mut arr[bound..=i], logger, slice_len - 1);
                if r == slice_len {
                    break;
                }
                stack.push(bound + r);
            }
        }
    }
}

// MovingPivot is excluded — see cyclent_sort.rs for rationale.
combo_codegen::sort_family!(
    type = CyclentSortStackOptimized<{P}>,
    uses = [
        "crate::sorts::quick_sorts::partitions::{Block, Hoare, Lomuto, ThreeWay}",
        "super::cyclent_sort_stack_optimized::CyclentSortStackOptimized",
    ],
    P: inline [
        ("Lomuto", "lomuto"),
        ("Hoare", "hoare"),
        ("ThreeWay", "three-way"),
        ("Block", "block"),
    ],
    name = "cyclent sort stack optimized",
    big_o = "O(N log N)",
    stable = false,
    direct_sort = true,
    path = ["fun sorts", "cyclent sort stack optimized", "{P}"],
);
