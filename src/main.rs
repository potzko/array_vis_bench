use std::time::Instant;

use crate::{sorts::insertion_sort::InsertionSort, traits::sort_traits::SortAlgo};

mod sorts;
mod traits;
mod utils;
fn main() {
    let mut arr = utils::array_gen::get_rand_arr(1000);
    //println!("{arr:?}");
    let mut logger = traits::log_traits::VisualizerLogger {
        log: Vec::<traits::log_traits::SortLog>::new(),
    };
    let start = Instant::now();
    sorts::bubble_sort::BubbleSort::sort(&mut arr, &mut logger);
    println!("{:?}", start.elapsed());
    //println!("{:?}", logger);
    println!("{:?}", logger.log.len());
    //println!("{arr:?}");
}

/*
fn main() {
    let mut arr = vec![3, 2, 1];
    let mut logger = traits::log_traits::VisualizerLogger {
        log: Vec::<traits::log_traits::SortLog>::new(),
    };
    InsertionSort::sort(&mut arr, &mut logger);
}*/
