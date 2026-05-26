//! `PartitionScheme` — partition algorithm role.
//!
//! Implemented by leaf crates like `partition_lomuto` so they can stay
//! tiny and reference only this trait crate + `sort_logger`. The wiring
//! crate (`array_vis_bench`) consumes the trait via its
//! `#[package.metadata.array_vis_bench.components]` cross-product.

use sort_logger::SortLogger;

pub trait PartitionScheme {
    /// Display name used both in the `Partition` component slot and in
    /// the per-algorithm path the menu builds at startup.
    const NAME: &'static str;
    /// Partition `arr` with the pivot originally at `pivot_idx`.
    ///
    /// Returns `(left_end, right_start)`:
    /// - `arr[..left_end]` needs further sorting
    /// - `arr[right_start..]` needs further sorting
    /// - `arr[left_end..right_start]` is already placed
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize);
}
