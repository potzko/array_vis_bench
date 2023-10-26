use crate::traits::sort_traits::SortAlgo;
use std::env;
use std::time::Instant;

mod rotations;
mod sorts;
mod traits;
mod utils;
mod visualise;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let size = 1000;
    let mut arr: Vec<usize> = utils::array_gen::get_rand_arr_in_range(size, 0, size);
    let mut _arr: Vec<usize> = utils::array_gen::get_arr(size);

    let original_arr = arr.clone();
    //println!("{arr:?}");
    let mut logger = traits::log_traits::VisualizerLogger {
        log: Vec::<traits::log_traits::SortLog<usize>>::new(),
        type_ghost: std::marker::PhantomData,
    };
    let start: Instant = Instant::now();
    sorts::merge_sorts::rotate_merge_sorts::rotate_merge_sort_optimized::MergeSort::sort(
        &mut arr,
        &mut (logger),
    );
    println!("{:?}", start.elapsed());
    //println!("{:?}", logger);
    println!("{:?}", logger.log.len());
    //println!("{:?}", &arr);
    println!("{}", utils::check_utils::is_sorted(&arr));
    println!(
        "{}",
        utils::check_utils::is_sorted_arr(&arr, &mut original_arr.clone())
    );

    visualise::img_tmp::main(&original_arr, arr.as_ptr() as usize, &logger.log);
    //visualise::tmp::main(logger.log, &original_arr);
}
