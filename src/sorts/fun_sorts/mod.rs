pub mod bad_heap_sort;
pub mod bad_heap_sort_alt;
pub mod cyclent_sort;
pub mod cyclent_sort_stack;
pub mod cyclent_sort_stack_optimized;
pub mod random_shell_sort;
pub mod slow_sort;
pub mod stooge_sort;
pub mod cyclent_sort_opt;
pub mod slow_sort_potzko;
pub mod quick_surrender;
pub mod quick_surrender_optimised;

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/fun_sorts_combinations.rs"));
}

