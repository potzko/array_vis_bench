//! Shell sort family: `ShellSort` + `ShellSortOrdered` generic over a
//! [`sequences::GapSequence`], with nine concrete sequences.
//!
//! The catalog emits every (algorithm x sequence) pair from `shell.spec` and
//! `shell.avbs`.

pub mod sequences;
pub mod shell_sort;
pub mod shell_sort_ordered;

pub use sequences::{
    Ciura, ClassicGaps, GapSequence, Hibbard, Knuth, Optimized256, Pratt, Sedgewick,
    SedgewickBranching, Tokuda,
};
pub use shell_sort::ShellSort;
pub use shell_sort_ordered::ShellSortOrdered;
