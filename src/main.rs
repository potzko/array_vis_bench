use crate::{
    sorts::insertion_sort::InsertionSort,
    traits::{log_traits::VisualizerLogger, sort_traits::SortAlgo},
};

mod sorts;
mod traits;
fn main() {
    let tmp = traits::log_traits::SortLog::Cmp {
        name: 1,
        ind_a: 2,
        ind_b: 3,
        result: true,
    };
    println!("{:?}", tmp);
    let mut arr = vec![1, 2, 3, 4, 5, 4, 3, 2, 1];
    let mut logger = traits::log_traits::VisualizerLogger {
        log: Vec::<traits::log_traits::SortLog>::new(),
    };
    InsertionSort::sort(&mut arr, &mut logger);
    println!("{arr:?}\n{logger:?}");
}
