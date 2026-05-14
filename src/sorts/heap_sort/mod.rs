pub mod arity;
pub mod arity_heap;
pub mod compare;
pub mod deep_heapify;
pub mod direction;
pub mod heap;
pub mod heap_algorithm;
pub mod heap_partition;
pub mod heap_sort;
pub mod heap_sort_quick_build;
pub mod layout;
pub mod quick_deep_heapify;

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/heap_sort_combinations.rs"));
}

