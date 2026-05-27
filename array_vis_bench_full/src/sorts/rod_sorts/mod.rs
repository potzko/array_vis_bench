//! Re-export shim. `RodSort<S, M>`, `BranchingStrategy`, the rod-merge
//! variants, and the ctor that populates `sort_registry_core` all live
//! in `rod_sort_lib`.

pub mod branching {
    pub use rod_sort_lib::branching::*;
}
pub mod merge {
    pub use rod_sort_lib::merge::*;
}
pub mod rod_sort {
    pub use rod_sort_lib::rod_sort::*;
}
