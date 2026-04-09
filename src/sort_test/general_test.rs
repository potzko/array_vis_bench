use crate::bench_registry;

pub fn test_sort(choice: &[String]) -> bool {
    // Find the sort by name in BENCH_SORTS
    let name = choice.last().map(String::as_str).unwrap_or("");
    for entry in bench_registry::BENCH_SORTS.iter() {
        if entry.name == name {
            return test_entry(entry);
        }
    }
    eprintln!("sort_test: '{}' not found in BENCH_SORTS", name);
    false
}

pub fn test_all() -> bool {
    let mut all_ok = true;
    for entry in bench_registry::BENCH_SORTS.iter() {
        if !test_entry(entry) {
            all_ok = false;
        }
    }
    all_ok
}

fn test_entry(entry: &bench_registry::SortBenchEntry) -> bool {
    let cases: Vec<(&str, Vec<usize>)> = vec![
        ("empty", vec![]),
        ("single", vec![1]),
        ("reversed pair", vec![2, 1]),
        ("sorted pair", vec![1, 2]),
        ("reversed 32", (0..32).rev().collect()),
        ("sorted 32", (0..32).collect()),
        ("all-same 32", vec![5; 32]),
        ("alternating 33", (0..33).map(|i| if i % 2 == 0 { i } else { 33 - i }).collect()),
        ("reversed 100", (0..100).rev().collect()),
    ];

    let mut ok = true;
    for (label, case) in &cases {
        let mut arr = case.clone();
        let mut expected = case.clone();
        expected.sort();
        (entry.run)(&mut arr);
        if arr != expected {
            eprintln!("{}: FAILED on '{}'", entry.name, label);
            ok = false;
        }
    }
    if ok {
        eprintln!("{}: OK", entry.name);
    }
    ok
}
