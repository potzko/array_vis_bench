//! Stub component types shaped like the real ones, plus both front-ends in use.
//!
//! Pivot is NESTED under the partition (the agreed layout), and the slot ROLE
//! enforces pivot arity: `LeftLeftPartition<V: SinglePivot>` cannot be built
//! with a `DualPivot` selector — rustc rejects it, and the engine rejects it
//! even earlier (see spec_core's `arity_violation_is_rejected_by_the_engine`).

use std::marker::PhantomData;

// ── role traits ───────────────────────────────────────────────────────────────
pub trait SinglePivot {
    const NAME: &'static str;
}
pub trait DualPivot {
    const NAME: &'static str;
}
pub trait Partition {
    const NAME: &'static str;
}
pub trait SmallSort {
    const THRESHOLD: usize;
}

// ── pivot selectors ───────────────────────────────────────────────────────────
pub struct FirstElement;
impl SinglePivot for FirstElement {
    const NAME: &'static str = "first";
}
pub struct MiddleElement;
impl SinglePivot for MiddleElement {
    const NAME: &'static str = "mid";
}
pub struct MedianOfThree;
impl SinglePivot for MedianOfThree {
    const NAME: &'static str = "med3";
}
pub struct TukeyNinther;
impl DualPivot for TukeyNinther {
    const NAME: &'static str = "tukey";
}

// ── partitions: each owns its pivot; the bound encodes arity ──────────────────
pub struct LeftLeftPartition<V: SinglePivot>(PhantomData<V>);
impl<V: SinglePivot> Partition for LeftLeftPartition<V> {
    const NAME: &'static str = "LL";
}
pub struct HoarePartition<V: SinglePivot>(PhantomData<V>);
impl<V: SinglePivot> Partition for HoarePartition<V> {
    const NAME: &'static str = "LR";
}
pub struct DualPivotPartition<V: DualPivot>(PhantomData<V>);
impl<V: DualPivot> Partition for DualPivotPartition<V> {
    const NAME: &'static str = "dual";
}

// ── small sorts ─────────────────────────────────────────────────────────────
pub struct NoSmallSort;
impl SmallSort for NoSmallSort {
    const THRESHOLD: usize = 0;
}
pub struct InsertionSmallSort<const N: usize>;
impl<const N: usize> SmallSort for InsertionSmallSort<N> {
    const THRESHOLD: usize = N;
}

// ── the sort ─────────────────────────────────────────────────────────────────
pub struct QuickSort<P: Partition, SS: SmallSort>(PhantomData<(P, SS)>);
impl<P: Partition, SS: SmallSort> QuickSort<P, SS> {
    pub fn sort(arr: &mut [usize]) {
        // Touch the params so the composition is genuinely instantiated.
        let _ = (P::NAME, <SS as SmallSort>::THRESHOLD);
        arr.sort_unstable();
    }
}

// ── mode 1: one tree, inline ──────────────────────────────────────────────────
spec_macro::sort_spec!(QuickLLMidIns32 = quick_sort<
    small_sort = insertion_sort<32>
    partition  = LL_partition< pivot = middle_element >
>);

// ── mode 2: many trees, generated at build time into `generated::SORTS` ───────
include!(concat!(env!("OUT_DIR"), "/generated_sorts.rs"));
