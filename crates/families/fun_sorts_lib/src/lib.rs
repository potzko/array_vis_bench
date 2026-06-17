//! Fun sorts — a grab bag of pedagogical and adversarial algorithms.
//! Family decls for the cross-product variants live in this crate's
//! `Cargo.toml`; the single-variant ones (`bad_heap_sort`,
//! `bad_heap_sort_alt`, `slow_sort`, `slow_sort_potzko`) self-register
//! via inline `sort_registry_macro::sort_family!` calls in their source
//! files — gated behind the default-off `self_register` feature so the spec
//! catalog can link this crate types-only and emit the entries itself (without
//! the runtime self-registration that would duplicate them).

/// Compositional complexity annotations (`HasTimeBounds`/`HasSpace`/
/// `HasStability`) for every fun-sort type — what the spec emit reads. Always
/// compiled (additive to the legacy `sort_family!` string `big_o`).
pub mod composable;

pub mod bad_heap_sort;
pub mod bad_heap_sort_alt;
pub mod cyclent_sort;
pub mod cyclent_sort_opt;
pub mod cyclent_sort_stack;
pub mod cyclent_sort_stack_optimized;
pub mod quick_surrender;
pub mod quick_surrender_optimised;
pub mod random_shell_sort;
pub mod slow_sort;
pub mod slow_sort_potzko;
pub mod stooge_sort;

pub use bad_heap_sort::BadHeapSort;
pub use bad_heap_sort_alt::BadHeapSortAlt;
pub use cyclent_sort::CyclentSort;
pub use cyclent_sort_opt::CyclentSortOpt;
pub use cyclent_sort_stack::CyclentSortStack;
pub use cyclent_sort_stack_optimized::CyclentSortStackOptimized;
pub use quick_surrender::QuickSurrender;
pub use quick_surrender_optimised::QuickSurrenderOptimised;
pub use random_shell_sort::{ParabolicDist, RandomShellSort, UniformDist};
pub use slow_sort::SlowSort;
pub use slow_sort_potzko::SlowSortPotzko;
pub use stooge_sort::StoogeSort;
