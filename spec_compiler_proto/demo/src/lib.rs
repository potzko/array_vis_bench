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

pub mod heap_lib {
    // A d-ary heap sort generic over a CONST arity. The registry only checks
    // that the arity is in a declared value set; `K >= 2` (a relation between
    // numbers) is rustc's job — here a const assertion stands in for it.
    pub struct HeapSort<const K: usize>;
    impl<const K: usize> HeapSort<K> {
        pub fn sort(arr: &mut [usize]) {
            // The value-level guard `K >= 2` is rustc's job (the second validity
            // layer), not the registry's — an inline const assertion stands in.
            const { assert!(K >= 2, "a d-ary heap needs arity >= 2") }
            arr.sort_unstable();
        }
    }
}

pub mod recursive_lib {
    // A recursive grammar: a sort whose `Inner` is itself a sort, bounded at
    // GENERATION time by the query depth knob (not by anything here).
    use super::*;
    pub trait RecSort {
        fn sort(arr: &mut [usize]);
    }
    pub struct BaseCase;
    impl RecSort for BaseCase {
        fn sort(arr: &mut [usize]) {
            arr.sort_unstable();
        }
    }
    impl BaseCase {
        pub fn sort(arr: &mut [usize]) {
            <Self as RecSort>::sort(arr)
        }
    }
    pub struct RecursiveSort<Inner>(PhantomData<Inner>);
    impl<Inner: RecSort> RecSort for RecursiveSort<Inner> {
        fn sort(arr: &mut [usize]) {
            Inner::sort(arr)
        }
    }
    impl<Inner: RecSort> RecursiveSort<Inner> {
        pub fn sort(arr: &mut [usize]) {
            <Self as RecSort>::sort(arr)
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

// ─────────────────────────────────────────────────────────────────────────────
// Phase 0 — de-hollow the center. Make the SORT stub types satisfy the REAL
// ABI (`avb_abi::{SortAlgo, HasTimeBounds, HasSpace, HasStability}`) so the
// generated `AlgorithmEntry` rows below type-check against the faithful ABI —
// not the old toy `(&str, fn(&mut [usize]))` table. The QuickSort impl keeps
// the arity where-clause, so an arity-mismatched QuickSort doesn't even
// implement `SortAlgo` (rustc backstop preserved).
// ─────────────────────────────────────────────────────────────────────────────

/// Shared stub body: announce the array, sort it, replay the final positions —
/// enough to drive any `SortLogger` and produce an observable event stream.
fn stub_drive<U: ?Sized + avb_abi::SortLogger<usize>>(arr: &mut [usize], logger: &mut U) {
    logger.create_array(arr.len());
    arr.sort_unstable();
    for (i, &v) in arr.iter().enumerate() {
        logger.write(i, v);
    }
}

impl<P, V, SS, U> avb_abi::SortAlgo<usize, U> for quick_sort_lib::quick_sort::QuickSort<P, V, SS>
where
    P: PartitionScheme,
    V: PivotInput<Arity = P::Arity>,
    SS: SmallSort,
    U: ?Sized + avb_abi::SortLogger<usize>,
{
    fn sort(arr: &mut [usize], logger: &mut U) {
        stub_drive(arr, logger);
    }
}
impl<P, V, SS> avb_abi::HasTimeBounds for quick_sort_lib::quick_sort::QuickSort<P, V, SS> {
    const WORST: avb_abi::Complexity = avb_abi::Complexity::from_str("O(n^2)");
    const AVERAGE: avb_abi::Complexity = avb_abi::Complexity::from_str("O(n log n)");
}
impl<P, V, SS> avb_abi::HasSpace for quick_sort_lib::quick_sort::QuickSort<P, V, SS> {
    const SPACE: avb_abi::Complexity = avb_abi::Complexity::from_str("O(log n)");
}
impl<P, V, SS> avb_abi::HasStability for quick_sort_lib::quick_sort::QuickSort<P, V, SS> {}

impl<S, const PP: bool, const EE: bool, U> avb_abi::SortAlgo<usize, U>
    for merge_sort_lib::top_down::TopDownMergeSort<S, PP, EE>
where
    S: SmallSort,
    U: ?Sized + avb_abi::SortLogger<usize>,
{
    fn sort(arr: &mut [usize], logger: &mut U) {
        stub_drive(arr, logger);
    }
}
impl<S, const PP: bool, const EE: bool> avb_abi::HasTimeBounds
    for merge_sort_lib::top_down::TopDownMergeSort<S, PP, EE>
{
    const WORST: avb_abi::Complexity = avb_abi::Complexity::from_str("O(n log n)");
}
impl<S, const PP: bool, const EE: bool> avb_abi::HasSpace
    for merge_sort_lib::top_down::TopDownMergeSort<S, PP, EE>
{
    const SPACE: avb_abi::Complexity = avb_abi::Complexity::from_str("O(n)");
}
impl<S, const PP: bool, const EE: bool> avb_abi::HasStability
    for merge_sort_lib::top_down::TopDownMergeSort<S, PP, EE>
{
    const STABLE: bool = true;
}

impl<Seq, U> avb_abi::SortAlgo<usize, U> for shell_sort_lib::shell_sort::ShellSort<Seq>
where
    Seq: GapSequence,
    U: ?Sized + avb_abi::SortLogger<usize>,
{
    fn sort(arr: &mut [usize], logger: &mut U) {
        stub_drive(arr, logger);
    }
}
impl<Seq> avb_abi::HasTimeBounds for shell_sort_lib::shell_sort::ShellSort<Seq> {
    const WORST: avb_abi::Complexity = avb_abi::Complexity::from_str("O(n^1.5)");
}
impl<Seq> avb_abi::HasSpace for shell_sort_lib::shell_sort::ShellSort<Seq> {
    const SPACE: avb_abi::Complexity = avb_abi::Complexity::from_str("O(1)");
}
impl<Seq> avb_abi::HasStability for shell_sort_lib::shell_sort::ShellSort<Seq> {}

impl<const K: usize, U> avb_abi::SortAlgo<usize, U> for heap_lib::HeapSort<K>
where
    U: ?Sized + avb_abi::SortLogger<usize>,
{
    fn sort(arr: &mut [usize], logger: &mut U) {
        stub_drive(arr, logger);
    }
}
impl<const K: usize> avb_abi::HasTimeBounds for heap_lib::HeapSort<K> {
    const WORST: avb_abi::Complexity = avb_abi::Complexity::from_str("O(n log n)");
}
impl<const K: usize> avb_abi::HasSpace for heap_lib::HeapSort<K> {
    const SPACE: avb_abi::Complexity = avb_abi::Complexity::from_str("O(1)");
}
impl<const K: usize> avb_abi::HasStability for heap_lib::HeapSort<K> {}

// The REAL emit target: AlgorithmEntry rows registered into avb_abi::ALGORITHMS.
include!(concat!(env!("OUT_DIR"), "/generated_entries.rs"));

#[cfg(test)]
mod entry_tests {
    // "implementations + a program = one program" — now true against the real
    // ABI shape, not a toy table.
    #[test]
    fn algorithms_slice_populated_and_names_unique() {
        let names: Vec<&str> = avb_abi::ALGORITHMS.iter().map(|e| e.name).collect();
        assert!(names.len() >= 30, "expected the SORT families, got {}", names.len());
        let uniq: std::collections::HashSet<_> = names.iter().collect();
        assert_eq!(uniq.len(), names.len(), "duplicate AlgorithmEntry names");
    }

    #[test]
    fn every_entry_runs_drives_logger_and_passes_correctness() {
        let cfg = avb_abi::RunConfig { size: 48, seed: 7 };
        for e in avb_abi::ALGORITHMS {
            let mut log = avb_abi::CaptureLogger::default();
            (e.run_with_input)("random", &cfg, &mut log);
            assert!(!log.events.is_empty(), "`{}` emitted no events", e.name);
            (e.run_correctness)(); // panics on failure
        }
    }

    #[test]
    fn complexity_is_inherited_from_the_type() {
        for e in avb_abi::ALGORITHMS {
            assert_ne!(e.worst, avb_abi::Complexity::UNKNOWN, "`{}` has no WORST", e.name);
        }
    }

    #[test]
    fn categories_and_flags_are_carried() {
        for e in avb_abi::ALGORITHMS {
            assert_eq!(e.category, avb_abi::Category::Sort);
        }
        // merge declared `adaptive true` in the catalog (a per-family literal).
        assert!(avb_abi::ALGORITHMS
            .iter()
            .any(|e| e.name.starts_with("merge[") && e.adaptive));
    }
}
