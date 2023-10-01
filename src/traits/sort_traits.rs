pub trait SortAlgo {
    fn max_size(&self) -> usize;
    fn big_o(&self) -> &str;
    fn name(&self) -> &str;
    fn sort<T: Ord + Copy, U: super::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U);
}
