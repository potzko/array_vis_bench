mod utils;
pub mod auxiliary_merge;
pub mod bottom_up;
pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/merge_sorts_combinations.rs"));
}
pub mod naive;
pub mod natural;
pub mod rotation;
pub mod rotation_merge;
pub mod standalone_registry;
pub mod top_down;

