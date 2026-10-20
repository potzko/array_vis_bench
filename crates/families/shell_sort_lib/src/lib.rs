//! Shell sort family — `ShellSort` + `ShellSortOrdered` generic over a
//! [`sequences::GapSequence`], with nine concrete sequences registered.
//!
//! The catalog emits every (algorithm x sequence) pair from `shell.spec` and
//! `shell.avbs`. The `self_register` feature, off by default and enabled
//! nowhere, keeps the older in-crate registration in [`registration`].

pub mod registration;
pub mod sequences;
pub mod shell_sort;
pub mod shell_sort_ordered;

pub use registration::{GapSequenceEntry, SortFn, GAP_SEQUENCES};
pub use sequences::{
    Ciura, ClassicGaps, GapSequence, Hibbard, Knuth, Optimized256, Pratt, Sedgewick,
    SedgewickBranching, Tokuda,
};
pub use shell_sort::ShellSort;
pub use shell_sort_ordered::ShellSortOrdered;
