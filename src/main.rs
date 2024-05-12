mod rotations;
mod sort_test;
mod sorts;
mod traits;
mod utils;
use std::time::*;
use utils::{array_gen::get_rand_arr, *};

mod visualise;

const TEST: bool = false;
const VIS: bool = true;
const BENCH: bool = false;

fn main() {
    //use std::env;
    //env::set_var("RUST_BACKTRACE", "1");

    if BENCH {
        let choice = pick_sort();
        let choice_2 = pick_sort();

        for i in [
            5, 10, 50, 100, 500, 1000, 5000, 10000, 50000, 100000, 500000, 1000000, 10000000,
        ] {
            let mut total = Duration::ZERO;
            for _ in 0..5 {
                let mut arr = get_rand_arr(i);
                let start = Instant::now();
                crate::sorts::fn_sort(&mut arr, &mut {}, &choice);
                total += start.elapsed();
            }
            println!(
                "took {:?} on avrage to sort arrays of length {i}",
                total / 10
            );
        }
        for i in [
            5, 10, 50, 100, 500, 1000, 5000, 10000, 50000, 100000, 500000, 1000000, 10000000,
        ] {
            let mut total = Duration::ZERO;
            for _ in 0..5 {
                let mut arr = get_rand_arr(i);
                let start = Instant::now();
                crate::sorts::fn_sort(&mut arr, &mut {}, &choice_2);
                total += start.elapsed();
            }
            println!(
                "took {:?} on avrage to sort arrays of length {i}",
                total / 10
            );
        }
    }

    if VIS {
        println!("enter length of array");
        let size = read_num_stdin();
        let mut arr: Vec<usize> = utils::array_gen::get_rand_arr_in_range(size, 0, size * size);
        let mut _arr: Vec<usize> = utils::array_gen::get_reversed_arr(size);
        let mut _arr: Vec<usize> = utils::array_gen::get_arr(size);

        let mut logger = traits::log_traits::VisualizerLogger {
            log: Vec::<traits::log_traits::SortLog<usize>>::new(),
            type_ghost: std::marker::PhantomData,
        };
        visualise::visualise_sort(&mut arr, &mut logger, &pick_sort());
    }

    if TEST {
        test_all()
    }
}

fn test_all() {
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
