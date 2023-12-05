use std::env;

mod rotations;
mod sort_test;
mod sorts;
mod traits;
mod utils;
use utils::*;
mod visualise;

fn main() {
    println!("{:?}", file!());
    /*
    println!("enter length of array");
    let size = read_num_stdin();
    let mut arr: Vec<usize> = utils::array_gen::get_rand_arr_in_range(size, 0, size);
    let mut _arr: Vec<usize> = utils::array_gen::get_arr(size);
    let mut _arr: Vec<usize> = utils::array_gen::get_reversed_arr(size);

    let mut logger = traits::log_traits::VisualizerLogger {
        log: Vec::<traits::log_traits::SortLog<usize>>::new(),
        type_ghost: std::marker::PhantomData,
    };
    */
    let mut logger = traits::log_traits::VisualizerLogger {
        log: Vec::<traits::log_traits::SortLog<usize>>::new(),
        type_ghost: std::marker::PhantomData,
    };
    let mut arr: Vec<usize> = vec![2, 2, 1];
    let mut arr: Vec<usize> = utils::array_gen::get_rand_arr_in_range(100, 0, 10);

    visualise::visualise_sort(
        &mut arr,
        &mut logger,
        &["cycle_sorts".to_string(), "cycle_sort".to_string()],
    );

    let a = sorts::get_all_sorts();
    for i in a {
        println!("{i:?}");
        println!("{}", sort_test::test_sort(&i));
    }
}

fn pick_sort() -> Vec<String> {
    let mut choice = vec![];
    loop {
        let options = sorts::options(&choice);
        if options.is_empty() {
            break;
        }
        println!("{:?} pick a sort:", choice);
        for (ind, value) in options.iter().enumerate() {
            println!("{}: {:?}", ind + 1, value);
        }
        let pick = read_num_stdin();
        if pick == 0 {
            break;
        }
        choice.push(options[pick - 1].clone());
    }
    choice
}
