//! Rod sort family — `RodSort<S, M>` plus the `BranchingStrategy` trait
//! and its six concrete strategies, and two `RodMerge` variants
//! (insertion / aux). Self-registers every (strategy × merge) pair via
//! the macro-generated `static ALGO_ENTRY` blocks in
//! [`branching`].

pub mod branching;
pub mod merge;
pub mod rod_sort;
pub mod shell_branching;

pub use branching::{RodEntry, ROD_STRATEGIES};
pub use merge::{AuxMerge, InsertionMerge, RodMerge};
pub use rod_sort::RodSort;
pub use shell_branching::{
    BranchingStrategy, Classic, Fibonacci, LogParity, Optimised, Parity3, RootParity,
};

/// Iterates the ROD_STRATEGIES distributed slice at startup and
/// registers every variant into `sort_registry_core`'s menu tree.
#[ctor::ctor]
fn register_rod_sorts() {
    for entry in ROD_STRATEGIES {
        let full: Vec<&str> = std::iter::once("sorts").chain(entry.path.iter().copied()).collect();
        sort_registry_core::register_sort_path(entry.name, entry.big_o, false, &full);
    }
}
