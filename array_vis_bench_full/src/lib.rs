pub mod sort_test;
pub mod sorts;
pub mod traits;
pub mod utils;
pub mod bench_registry;

pub use array_vis_bench_core::{inputs, visualise};
pub use array_vis_bench_core::{register_nondeterministic, register_test_cap};

#[cfg(test)]
mod property_tests;

// Force-link the side-effect-only registry crates. Without these
// `#[used]` anchors the linker drops their object files (and the
// `#[linkme::distributed_slice]` ALGORITHMS entries + `#[ctor]` menu
// registrations inside them) because nothing else references their
// symbols.
#[used]
#[allow(dead_code)]
static _PARTITION_REGISTRY_ANCHOR: &() = &quick_partition_registry::LINK_ANCHOR;
#[used]
#[allow(dead_code)]
static _MERGE_REGISTRY_ANCHOR: &() = &merge_standalone_registry::LINK_ANCHOR;
#[used]
#[allow(dead_code)]
static _QUICK_SELECT_REGISTRY_ANCHOR: &() = &quick_select_registry::LINK_ANCHOR;
