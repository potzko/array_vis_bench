pub trait SortAlgo {
    fn max_size(&self) -> usize;
    fn big_o(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn sort<T: Ord + Copy, U: super::log_traits::SortLogger<T>>(
        &self,
        arr: &mut [T],
        logger: &mut U,
    );
}
