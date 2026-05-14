pub mod bubble_sort;
pub mod bubble_sort_recursive;
pub mod odd_even_bubble_sort;
pub mod shaker_sort;

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/bubble_sorts_combinations.rs"));
}

