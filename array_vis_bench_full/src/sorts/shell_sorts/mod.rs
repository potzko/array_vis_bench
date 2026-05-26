//! Re-export shim. The shell sort family — `ShellSort<Seq>`,
//! `ShellSortOrdered<Seq>`, the `GapSequence` trait, and nine concrete
//! sequences — lives in `shell_sort_lib`. That crate self-registers
//! every (algorithm × sequence) pair into
//! `array_vis_bench_core::ALGORITHMS`.

pub use shell_sort_lib::{
    Ciura, Classic, GapSequence, GapSequenceEntry, Hibbard, Knuth, Optimized256, Pratt, Sedgewick,
    SedgewickBranching, ShellSort, ShellSortOrdered, Tokuda, GAP_SEQUENCES,
};
