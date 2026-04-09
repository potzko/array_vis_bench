#![allow(dead_code)]
use rand::Rng;

pub fn get_rand_arr(length: usize) -> Vec<usize> {
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

pub fn get_rand_arr_in_range(length: usize, min: usize, max: usize) -> Vec<usize> {
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen_range(min..max)).collect()
}

pub fn get_arr(length: usize) -> Vec<usize> {
    (0..length).collect()
}

pub fn get_reversed_arr(length: usize) -> Vec<usize> {
    (0..length).rev().collect()
}
