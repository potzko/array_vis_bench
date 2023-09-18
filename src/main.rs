use crate::traits::sort_traits::SortAlgo;
use std::time::Instant;
use traits::log_traits::SortLog::*;
use traits::log_traits::SortLogger;

mod sorts;
mod traits;
mod utils;
mod visualise;

fn main() {
    let size = 3000;
    let mut arr: Vec<u64> = utils::array_gen::get_rand_arr(size);
    //arr = crate::utils::array_gen::get_arr(size);
    let original_arr = arr.clone();
    //println!("{arr:?}");
    let mut logger = traits::log_traits::VisualizerLogger {
        log: Vec::<traits::log_traits::SortLog<u64>>::new(),
        type_ghost: std::marker::PhantomData,
    };

    for i in 0..10000 {
        logger.log(Mark("".to_string()))
    }

    let start: Instant = Instant::now();
    sorts::circle_sorts::stooge_circle_sort::CircleSort::sort(&mut arr, 0, size, &mut (logger));
    println!("{:?}", start.elapsed());
    //println!("{:?}", logger);
    println!("{:?}", logger.log.len());
    //println!("{:?}", &arr);
    println!("{}", utils::check_utils::is_sorted(&arr));
    println!(
        "{}",
        utils::check_utils::is_sorted_arr(&arr, &mut original_arr.clone())
    );
    let cmp_count = logger
        .log
        .iter()
        .filter(|&i| match i {
            traits::log_traits::SortLog::Cmp { .. } => true,
            traits::log_traits::SortLog::CmpOuterData { .. } => true,
            _ => false,
        })
        .count();
    println!("cmp count: {}", cmp_count);

    let swap_counter = logger
        .log
        .iter()
        .filter(|&i| match i {
            traits::log_traits::SortLog::Swap { .. } => true,
            _ => false,
        })
        .count();
    println!("swap count: {}", swap_counter);
    visualise::tmp::main(logger.log, &original_arr);
}

/*
fn main() {
    let mut arr = vec![3, 2, 1];
    let mut logger = traits::log_traits::VisualizerLogger {
        log: Vec::<traits::log_traits::SortLog>::new(),
    };
    InsertionSort::sort(&mut arr, &mut logger);
}*/
