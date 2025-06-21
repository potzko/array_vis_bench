pub trait SortAlgo<T: Ord + Copy, U: super::log_traits::SortLogger<T>>
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
