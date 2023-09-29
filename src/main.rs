use crate::sorts::heap_sort::base_16_heap::HeapSort;
use crate::traits::log_traits::SortLog;
use crate::traits::sort_traits::SortAlgo;
use ffmpeg_next::log;
use std::time::Instant;
use traits::log_traits::SortLog::*;
use traits::log_traits::SortLogger;

mod sorts;
mod traits;
mod utils;
mod visualise;

fn main() {
    let size = 1000;
    let mut arr: Vec<usize> = utils::array_gen::get_rand_arr(size);
    //arr = crate::utils::array_gen::get_arr(size);
    let mut original_arr = arr.clone();
    //println!("{arr:?}");
    let mut logger = traits::log_traits::VisualizerLogger {
        log: Vec::<traits::log_traits::SortLog<usize>>::new(),
        type_ghost: std::marker::PhantomData,
    };

    for _ in 0..1 {
        logger.log(Mark("started sort".to_string()))
    }

    let start: Instant = Instant::now();
    sorts::quick_sorts::quick_sort_left_left_pointers::QuickSort::sort(
        &mut arr,
        0,
        size,
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
    let cmp_count = logger
        .log
        .iter()
        .filter(|&i| {
            matches!(
                i,
                traits::log_traits::SortLog::CmpInArr { .. }
                    | traits::log_traits::SortLog::CmpData { .. }
            )
        })
        .count();
    println!("cmp count: {}", cmp_count);

    let swap_counter = logger
        .log
        .iter()
        .filter(|&i| matches!(i, traits::log_traits::SortLog::Swap { .. },))
        .count();
    println!("swap count: {}", swap_counter);

    visualise::img_tmp::main(&original_arr, arr.as_ptr() as usize, &logger.log);
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
