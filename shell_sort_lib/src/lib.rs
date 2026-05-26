//! Shell sort family — `ShellSort` + `ShellSortOrdered` generic over a
//! [`sequences::GapSequence`], with nine concrete sequences registered.
//!
//! Self-registers into `array_vis_bench_core::ALGORITHMS` via
//! `#[linkme::distributed_slice]` entries in [`registration`], so any
//! binary that depends on this crate (directly or transitively) has the
//! full (algorithm × sequence) cross-product available at runtime.

pub mod registration;
pub mod sequences;
pub mod shell_sort;
pub mod shell_sort_ordered;

pub use registration::{GapSequenceEntry, SortFn, GAP_SEQUENCES};
pub use sequences::{
    Ciura, Classic, GapSequence, Hibbard, Knuth, Optimized256, Pratt, Sedgewick,
    SedgewickBranching, Tokuda,
};
pub use shell_sort::ShellSort;
pub use shell_sort_ordered::ShellSortOrdered;
