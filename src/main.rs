use std::time::Instant;

use crate::{sorts::insertion_sort::InsertionSort, traits::sort_traits::SortAlgo};

mod sorts;
mod traits;
mod utils;
mod visualise;
fn main() {
    let mut arr = utils::array_gen::get_rand_arr(2);
    //println!("{arr:?}");
    let mut logger = traits::log_traits::VisualizerLogger {
        log: Vec::<traits::log_traits::SortLog>::new(),
    };
    let start = Instant::now();
    sorts::bubble_sorts::odd_even_bubble_sort::Odd_Even_Bubble_Sort::sort(&mut arr, &mut ());
    println!("{:?}", start.elapsed());
    println!("{:?}", logger);
    println!("{:?}", logger.log.len());
    //println!("{arr:?}");
    print!("{}", utils::check_utils::is_sorted(&arr));
    visualise::tmp::main();
}

/*
fn main() {
    let mut arr = vec![3, 2, 1];
    let mut logger = traits::log_traits::VisualizerLogger {
        log: Vec::<traits::log_traits::SortLog>::new(),
    };
    InsertionSort::sort(&mut arr, &mut logger);
}*/
