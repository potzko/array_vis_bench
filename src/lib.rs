pub mod sort_test;
pub mod sorts;
pub mod traits;
pub mod utils;
pub mod visualise;
pub mod bench_registry;
pub mod inputs;

#[cfg(test)]
mod property_tests;

// Auto-registration is handled via derive macros with linkme; no generated registrations needed.
