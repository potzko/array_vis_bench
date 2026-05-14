pub mod comb_sort;
pub mod comb_sort_ratio;
pub mod register_sequences;
pub mod sequences;

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/comb_sorts_combinations.rs"));
}

