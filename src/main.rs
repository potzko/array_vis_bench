use std::time::Instant;

use crate::traits::sort_traits::SortAlgo;

mod sorts;
mod traits;
mod utils;
mod visualise;
fn main() {
    let size = 10000000;
    let mut arr: Vec<u64> = utils::array_gen::get_rand_arr(size);
    let original_arr = arr.clone();
    //println!("{arr:?}");
    let mut logger = traits::log_traits::VisualizerLogger {
        log: Vec::<traits::log_traits::SortLog<u64>>::new(),
        type_ghost: std::marker::PhantomData,
    };
    let start: Instant = Instant::now();
    sorts::heap_sort::heap_sort_classic::HeapSort::sort(&mut arr, 0, size, &mut (logger));
    println!("{:?}", start.elapsed());
    //println!("{:?}", logger);
    println!("{:?}", logger.log.len());
    //println!("{:?}", &arr);
    println!("{}", utils::check_utils::is_sorted(&arr));
    println!(
        "{}",
        utils::check_utils::is_sorted_arr(&arr, &mut original_arr.clone())
    );

    //visualise::tmp::main(logger.log, &original_arr);
}

/*
fn main() {
    let mut arr = vec![3, 2, 1];
    let mut logger = traits::log_traits::VisualizerLogger {
        log: Vec::<traits::log_traits::SortLog>::new(),
    };
    InsertionSort::sort(&mut arr, &mut logger);
}*/
