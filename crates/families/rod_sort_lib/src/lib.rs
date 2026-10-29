//! Rod sort family — `RodSort<S, M>` plus the `BranchingStrategy` trait
//! and its six concrete strategies, and three `RodMerge` variants
//! (insertion / aux / tournament). The catalog emits every (strategy x merge) pair from
//! `rod.spec` and `rod.avbs`.

pub mod gaps;
pub mod merge;
pub mod rod_sort;
pub mod shell_branching;
#[cfg(test)]
mod testing;

pub use gaps::RodGaps;
pub use merge::{AuxMerge, InsertionMerge, RodMerge, TournamentMerge};
pub use rod_sort::RodSort;
pub use shell_branching::{
    BranchingStrategy, Classic, Fibonacci, LogParity, Optimised, Parity3, RootParity,
};
