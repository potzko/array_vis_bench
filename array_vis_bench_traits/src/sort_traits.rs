//! Indirect-dispatch sort trait — `direct_sort = false` in `sort_family!`
//! routes through `<T as SortAlgo<...>>::sort(arr, logger)` instead of an
//! inherent method. Only a handful of sorts still use this path; new
//! algorithms should prefer an inherent `sort(arr, logger)` method.

use sort_logger::SortLogger;

pub trait SortAlgo<T: Ord + Copy, U: SortLogger<T>>
where
    Self: Sized,
{
    fn big_o() -> &'static str;
    fn name() -> &'static str;
    fn sort(arr: &mut [T], logger: &mut U);
    fn stable() -> bool;
    fn sort_obj(&self, arr: &mut [T], logger: &mut U) {
        Self::sort(arr, logger)
    }
}
