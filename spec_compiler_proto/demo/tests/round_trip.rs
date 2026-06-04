use demo::quick_sort_lib::pivot_selectors::CombinedSelector;
use demo::quick_sort_lib::quick_sort::QuickSort;
use demo::quick_sort_lib::yaroslavskiy::DualPivotPartition;
use demo::small_sort_insertion::InsertionSmallSort;
use demo::small_sort_insertion_strategy::BinaryInsertion;
use demo::*;
use std::marker::PhantomData;

fn assert_same_type<T>(_: PhantomData<T>) {}

#[test]
fn type_plus_const_and_imports_resolve() {
    // QuickSingle = QuickSort<LeftLeftPartition, MiddleElement, InsertionSmallSort<BinaryInsertion, 32>>
    assert_same_type::<QuickSingle>(PhantomData::<
        QuickSort<
            partition_lomuto::LeftLeftPartition,
            pivots::MiddleElement,
            InsertionSmallSort<BinaryInsertion, 32>,
        >,
    >);
    assert_eq!(QuickSingle_NAME, "quick[LL/mid/ins:bin:32]");
}

#[test]
fn arity_composed_dual_selector_resolves() {
    // QuickDual uses CombinedSelector<FirstElement, MiddleElement> on a dual partition.
    assert_same_type::<QuickDual>(PhantomData::<
        QuickSort<
            DualPivotPartition,
            CombinedSelector<pivots::FirstElement, pivots::MiddleElement>,
            small_sorts::NoSmallSort,
        >,
    >);
    assert_eq!(QuickDual_NAME, "quick[dual/combined<first,mid>/none]");
}

#[test]
fn bool_consts_resolve() {
    assert_eq!(MergePP_NAME, "merge[ins:lin:64/pp=true/ee=false]");
}

#[test]
fn all_inline_sorts_run() {
    for run in [QuickSingle_run, QuickDual_run, MergePP_run, ShellCiura_run] {
        let mut a = [5usize, 3, 8, 1, 2];
        run(&mut a);
        assert_eq!(a, [1, 2, 3, 5, 8]);
    }
}

#[test]
fn generated_table_runs() {
    // quick(21) + merge(3) + shell(3) + heap(3) + recursive(4) = 34.
    // The headline: the quick_sort family is emitted IN FULL. The old build.rs
    // had to drop it because flat enumeration overproduced arity-illegal combos;
    // here a shared pivot variable made them unrepresentable, so every row is
    // arity-correct — which is exactly why this whole module compiles.
    assert_eq!(generated::SORTS.len(), 34);
    for (name, run) in generated::SORTS {
        let mut a = [9usize, 1, 5, 3, 7, 2, 8, 4, 6, 0];
        run(&mut a);
        assert_eq!(a, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9], "sort `{name}` failed");
    }

    let names: Vec<&str> = generated::SORTS.iter().map(|(n, _)| *n).collect();
    assert_eq!(names.iter().filter(|n| n.starts_with("quick[")).count(), 21);
    assert!(names.iter().any(|n| n.starts_with("heap[")));
    assert!(names.iter().any(|n| n.starts_with("rec[")));
    // The arity invariant holds at the table level too: an LL partition never
    // carries a dual selector, and a dual partition never carries a single one.
    for n in &names {
        if n.starts_with("quick[LL/") {
            assert!(!n.contains("ninther") && !n.contains("combined"), "arity leak: {n}");
        }
        if n.starts_with("quick[dual/") {
            assert!(!n.contains("/first/") && !n.contains("/mid/"), "arity leak: {n}");
        }
    }
}
