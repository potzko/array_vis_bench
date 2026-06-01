//! Quick surrender — selection sort via quickselect.
//!
//! For each prefix slot `i`, run a quickselect over `arr[i..]` asking for
//! the smallest element (target = 0). The element is left at index `i`
//! and the rest of the slice is shuffled but otherwise unsorted —
//! traditional selection sort's `O(N^2)` outer loop, but each find-min
//! step is a recursive partition rather than a linear scan, so the
//! visualiser sees a partition-heavy trace.
//!
//! Parametrised over a [`QuickSelect`] strategy (recursive vs iterative)
//! which itself fans out over [`PartitionScheme`] × [`PivotSelector`].

use std::marker::PhantomData;

use array_vis_bench_traits::QuickSelect;
use sort_logger::SortLogger;

pub struct QuickSurrender<QS: QuickSelect>(PhantomData<QS>);

impl<QS: QuickSelect> QuickSurrender<QS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        for i in 0..n {
            QS::select(&mut arr[i..], logger, 0);
        }
    }
}

