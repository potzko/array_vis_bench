//! Quick surrender (optimised) — block quickselect plus small sort.
//!
//! Sweep the array left-to-right in fixed-size blocks of `SS::THRESHOLD`.
//! Each block: run `QS::select(arr[i..], block - 1)` so the block's slot
//! ends up holding the `block` smallest elements of the remaining tail
//! (unordered), then have `SS` sort that block in place. The block size
//! comes from the [`NonTrivialSmallSort`]'s threshold, so it's always ≥ 2.

use std::marker::PhantomData;

use array_vis_bench_traits::QuickSelect;
use sort_logger::SortLogger;
use array_vis_bench_traits::NonTrivialSmallSort;

pub struct QuickSurrenderOptimised<QS: QuickSelect, SS: NonTrivialSmallSort>(
    PhantomData<(QS, SS)>,
);

impl<QS: QuickSelect, SS: NonTrivialSmallSort> QuickSurrenderOptimised<QS, SS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        let block_size = SS::THRESHOLD;
        let mut i = 0;
        while i < n {
            let block = (n - i).min(block_size);
            QS::select(&mut arr[i..], logger, block - 1);
            SS::sort(&mut arr[i..i + block], logger);
            i += block;
        }
    }
}

