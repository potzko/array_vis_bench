use crate::traits::sort_traits::SortAlgo;
use std::env;
use std::time::Instant;

mod rotations;
mod sorts;
mod traits;
mod utils;
use utils::*;
mod visualise;
use strum::IntoEnumIterator; // 0.17.1

type LoggerChoice = traits::log_traits::VisualizerLogger<usize>;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    println!("enter length of array");
    let size = read_num_stdin();
    let mut arr: Vec<usize> = utils::array_gen::get_rand_arr_in_range(size, 0, size);
    let mut _arr: Vec<usize> = utils::array_gen::get_arr(size);
    let mut _arr: Vec<usize> = utils::array_gen::get_reversed_arr(size);

    let original_arr = arr.clone();
    //println!("{arr:?}");
    let mut logger = traits::log_traits::VisualizerLogger {
        log: Vec::<traits::log_traits::SortLog<usize>>::new(),
        type_ghost: std::marker::PhantomData,
    };
    for i in sorts::AnySort::iter() {
        println!("{:?}", i);
    }
    let sort_choice = sorts::fun_sorts::random_shell_sort::FunSort {};
    let start: Instant = Instant::now();
    sort_choice.sort(&mut arr, &mut (logger));
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
