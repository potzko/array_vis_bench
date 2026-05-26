pub mod sort_test;
pub mod sorts;
pub mod traits;
pub mod utils;
pub mod bench_registry;

pub use array_vis_bench_core::{inputs, visualise};
pub use array_vis_bench_core::{register_nondeterministic, register_test_cap};

#[cfg(test)]
mod property_tests;

// Auto-registration is handled via derive macros with linkme; no generated registrations needed.
