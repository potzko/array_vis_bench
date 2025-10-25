pub mod benchmark;
pub mod rotations;
pub mod sort_test;
pub mod sorts;
pub mod traits;
pub mod utils;
pub mod visualise;

// Include the generated registrations as a module
pub mod generated_registrations {
    include!("../generated_registrations.rs");
}
