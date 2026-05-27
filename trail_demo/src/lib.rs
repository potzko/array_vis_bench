//! Isolated fixture proving the combo_codegen trail rule terminates a
//! mutually-recursive component graph and emits only compilable types.
//!
//! The traits and structs are intentionally empty — they exist solely so
//! the generated `DemoSort<…>` aliases (see the included file) have real
//! types to bind, type-checking every variant the trail-bounded expansion
//! produced in `build.rs`.

#![allow(dead_code)]

use core::marker::PhantomData;

pub trait Partition {}
pub trait HeapBuild {}

// ── Leaf partitions ──
pub struct Lomuto;
impl Partition for Lomuto {}

pub struct Hoare;
impl Partition for Hoare {}

// ── Leaf build ──
pub struct SimpleBuild;
impl HeapBuild for SimpleBuild {}

// ── Composite partition: heap-extraction, generic over how the heap is built ──
pub struct HeapExtract<B: HeapBuild>(PhantomData<B>);
impl<B: HeapBuild> Partition for HeapExtract<B> {}

// ── Composite build: quickselect, generic over the partition it uses ──
// This closes the cycle: Partition → HeapBuild → Partition → …
pub struct QuickBuild<P: Partition>(PhantomData<P>);
impl<P: Partition> HeapBuild for QuickBuild<P> {}

// ── Root family wrapper ──
pub struct DemoSort<P: Partition>(PhantomData<P>);

// Trail-bounded enumeration emitted by build.rs: `COUNT`, `VARIANT_TYPES`,
// and a `_typecheck` module of `type V{i} = DemoSort<…>;` aliases.
include!(concat!(env!("OUT_DIR"), "/trail_demo_variants.rs"));

#[cfg(test)]
mod tests {
    use super::{COUNT, VARIANT_TYPES};

    #[test]
    fn expansion_terminates_at_six_variants() {
        // 2 leaf partitions + HeapExtract over each non-looping HeapBuild.
        assert_eq!(COUNT, 6);
        assert_eq!(VARIANT_TYPES.len(), 6);
    }

    #[test]
    fn includes_depth_one_nesting() {
        assert!(VARIANT_TYPES.contains(&"DemoSort<Lomuto>"));
        assert!(VARIANT_TYPES.contains(&"DemoSort<HeapExtract<SimpleBuild>>"));
        assert!(VARIANT_TYPES.contains(&"DemoSort<HeapExtract<QuickBuild<Lomuto>>>"));
    }

    #[test]
    fn deepest_renest_bottoms_out_in_a_leaf_build() {
        // HeapExtract may recur once (via a different parent edge), but its
        // inner build is forced to SimpleBuild because the
        // (HeapExtract, B, QuickBuild) edge is already on the path.
        assert!(VARIANT_TYPES
            .contains(&"DemoSort<HeapExtract<QuickBuild<HeapExtract<SimpleBuild>>>>"));
    }

    #[test]
    fn prunes_the_looping_edge() {
        // The cycle QuickBuild→HeapExtract→QuickBuild must never appear.
        assert!(
            !VARIANT_TYPES
                .iter()
                .any(|t| t.contains("QuickBuild<HeapExtract<QuickBuild")),
            "looped edge leaked: {VARIANT_TYPES:?}",
        );
    }
}
