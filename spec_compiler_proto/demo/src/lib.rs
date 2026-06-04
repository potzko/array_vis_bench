//! Stub types mirroring the REAL crate module paths and signatures (from the
//! survey), so the engine's imports + const handling + arity are stressed
//! against realistic shapes — not shapes bent to fit the engine.
//!
//! Arity is enforced the way the real code does it: `QuickSort<P, V, SS>` keeps
//! pivot `V` as a SIBLING of partition `P`, and a where-clause ties their
//! arities via associated-type equality (`V: PivotInput<Arity = P::Arity>`), so
//! a single-pivot partition + dual selector fails to compile.

#![allow(dead_code)]
use std::marker::PhantomData;

// arity tags
pub struct One;
pub struct Two;

pub trait PartitionScheme {
    type Arity;
}
pub trait PivotInput {
    type Arity;
}
pub trait SmallSort {
    const THRESHOLD: usize;
}
pub trait InsertionStrategy {}
pub trait GapSequence {}

// ── module paths mirroring the real crates ────────────────────────────────────
pub mod partition_lomuto {
    use super::*;
    pub struct LeftLeftPartition;
    impl PartitionScheme for LeftLeftPartition {
        type Arity = One;
    }
}

pub mod pivots {
    use super::*;
    pub struct FirstElement;
    impl PivotInput for FirstElement {
        type Arity = One;
    }
    pub struct MiddleElement;
    impl PivotInput for MiddleElement {
        type Arity = One;
    }
}

pub mod quick_sort_lib {
    use super::*;
    pub mod yaroslavskiy {
        use super::*;
        pub struct DualPivotPartition;
        impl PartitionScheme for DualPivotPartition {
            type Arity = Two;
        }
    }
    pub mod pivot_selectors {
        use super::*;
        pub struct NintherDualPivot;
        impl PivotInput for NintherDualPivot {
            type Arity = Two;
        }
        // dual selector composed from two single selectors
        pub struct CombinedSelector<V1, V2>(PhantomData<(V1, V2)>);
        impl<V1, V2> PivotInput for CombinedSelector<V1, V2>
        where
            V1: PivotInput<Arity = One>,
            V2: PivotInput<Arity = One>,
        {
            type Arity = Two;
        }
    }
    pub mod quick_sort {
        use super::*;
        pub struct QuickSort<P, V, SS>(PhantomData<(P, V, SS)>);
        impl<P, V, SS> QuickSort<P, V, SS>
        where
            P: PartitionScheme,
            V: PivotInput<Arity = P::Arity>, // <- arity must match
            SS: SmallSort,
        {
            pub fn sort(arr: &mut [usize]) {
                let _ = <SS as SmallSort>::THRESHOLD;
                arr.sort_unstable();
            }
        }
    }
}

pub mod small_sorts {
    use super::*;
    pub struct NoSmallSort;
    impl SmallSort for NoSmallSort {
        const THRESHOLD: usize = 0;
    }
}

pub mod small_sort_insertion_strategy {
    use super::*;
    pub struct LinearInsertion;
    impl InsertionStrategy for LinearInsertion {}
    pub struct BinaryInsertion;
    impl InsertionStrategy for BinaryInsertion {}
}

pub mod small_sort_insertion {
    use super::*;
    // TYPE param (strategy) + CONST generic (threshold)
    pub struct InsertionSmallSort<S, const N: usize>(PhantomData<S>);
    impl<S: InsertionStrategy, const N: usize> SmallSort for InsertionSmallSort<S, N> {
        const THRESHOLD: usize = N;
    }
}

pub mod merge_sort_lib {
    use super::*;
    pub mod top_down {
        use super::*;
        // TYPE param + TWO bool consts
        pub struct TopDownMergeSort<S, const PING_PONG: bool, const EARLY_EXIT: bool>(PhantomData<S>);
        impl<S: SmallSort, const PING_PONG: bool, const EARLY_EXIT: bool>
            TopDownMergeSort<S, PING_PONG, EARLY_EXIT>
        {
            pub fn sort(arr: &mut [usize]) {
                let _ = (<S as SmallSort>::THRESHOLD, PING_PONG, EARLY_EXIT);
                arr.sort_unstable();
            }
        }
    }
}

pub mod shell_sort_lib {
    use super::*;
    pub mod sequences {
        use super::*;
        pub struct Classic;
        impl GapSequence for Classic {}
        pub struct Knuth;
        impl GapSequence for Knuth {}
        pub struct Ciura;
        impl GapSequence for Ciura {}
    }
    pub mod shell_sort {
        use super::*;
        pub struct ShellSort<Seq>(PhantomData<Seq>);
        impl<Seq: GapSequence> ShellSort<Seq> {
            pub fn sort(arr: &mut [usize]) {
                arr.sort_unstable();
            }
        }
    }
}

// ── mode 1: inline one-offs. All three exercise imports; the consts and
//    type+const combos go through the generalized const path. ────────────────

// single partition + single pivot + (binary insertion, threshold 32) — LEGAL
spec_macro::sort_spec!(QuickSingle = quick_sort<
    partition  = LL_partition
    pivot      = middle_element
    small_sort = insertion< strategy = binary, 32 >
>);

// dual partition + dual (combined) selector — LEGAL (arities match)
spec_macro::sort_spec!(QuickDual = quick_sort<
    partition = dual_pivot_partition
    pivot     = combined< a = first_element, b = middle_element >
    small_sort = no_small_sort
>);

// merge sort with both bool consts set by name
spec_macro::sort_spec!(MergePP = top_down_merge<
    small_sort = insertion< 64 >
    ping_pong  = true
    early_exit = false
>);

// shell sort
spec_macro::sort_spec!(ShellCiura = shell_sort< seq = ciura >);

// ── mode 2: build-time enumeration (merge + shell only; see build.rs) ─────────
include!(concat!(env!("OUT_DIR"), "/generated_sorts.rs"));
