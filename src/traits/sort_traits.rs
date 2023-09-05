pub trait sort_algo<T: Ord, U: super::log_traits::SortLogger> {
    fn max_size(&self) -> usize;
    fn big_o(&self) -> &str;
    fn sort(arr: &mut [T], logger: &mut U);
}
