use demo::*;
use std::marker::PhantomData;

/// Compile-time proof the alias resolved to the EXACT concrete type expected.
fn assert_same_type<T>(_: PhantomData<T>) {}

#[test]
fn inline_macro_resolves_to_concrete_type() {
    assert_same_type::<QuickLLMidIns32>(PhantomData::<
        QuickSort<LeftLeftPartition<MiddleElement>, InsertionSmallSort<32>>,
    >);
    assert_eq!(QuickLLMidIns32_NAME, "quick[LL<mid>/ins:32]");
    let mut a = [5usize, 3, 8, 1, 2];
    QuickLLMidIns32_run(&mut a);
    assert_eq!(a, [1, 2, 3, 5, 8]);
}

#[test]
fn generated_table_has_every_legal_sort() {
    // 7 partitions × 2 small sorts
    assert_eq!(generated::SORTS.len(), 14);
}

#[test]
fn every_generated_sort_runs() {
    for (name, run) in generated::SORTS {
        let mut a = [9usize, 1, 5, 3, 7, 2, 8, 4, 6, 0];
        run(&mut a);
        assert_eq!(a, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9], "sort `{name}` failed");
    }
}

#[test]
fn generated_labels_respect_arity() {
    let names: Vec<&str> = generated::SORTS.iter().map(|(n, _)| *n).collect();
    assert!(names.contains(&"quick[dual<tukey>/none]"));
    assert!(names.iter().any(|n| n.starts_with("quick[LL<mid>")));
    // dual selector never paired with a single-pivot partition, nor vice-versa
    assert!(!names.iter().any(|n| n.contains("LL<tukey>") || n.contains("dual<mid>")));
}
