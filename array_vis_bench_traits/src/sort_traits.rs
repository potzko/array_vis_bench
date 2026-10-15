//! Indirect-dispatch sort trait: routes through
//! `<T as SortAlgo<...>>::sort(arr, logger)` instead of an inherent method.
//! Only a handful of sorts still use this path; new algorithms should expose
//! an inherent `sort(arr, logger)` method, which is the shape the spec emit
//! requires.

use sort_logger::SortLogger;

/// Trait-routed sort dispatch.
///
/// New sorts should prefer an inherent `sort(arr, logger)` method; this
/// trait is kept for the small set of legacy variants that only implement
/// it.
pub trait SortAlgo<T: Ord + Copy, U: SortLogger<T>>
where
    Self: Sized,
{
    /// Human-readable big-O class, e.g. `"O(N log N)"`. Used by the
    /// menu / registry layer; not consulted by the algorithm itself.
    fn big_o() -> &'static str;
    /// Stable sort identifier — must match the corresponding
    /// `AlgorithmEntry.name` so the registry can resolve back-references.
    fn name() -> &'static str;
    /// In-place sort. Calls `logger` for every observable operation.
    fn sort(arr: &mut [T], logger: &mut U);
    /// Whether the sort preserves the relative order of equal keys.
    fn stable() -> bool;
    /// Object-style dispatch (forwards to [`Self::sort`]); useful when
    /// the caller holds a trait object rather than a static type.
    fn sort_obj(&self, arr: &mut [T], logger: &mut U) {
        Self::sort(arr, logger)
    }
}
