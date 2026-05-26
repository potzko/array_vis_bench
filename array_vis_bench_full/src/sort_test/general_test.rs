use crate::bench_registry::{self, AlgorithmEntry, Category};

pub fn test_sort(choice: &[String]) -> bool {
    let name = choice.last().map(String::as_str).unwrap_or("");
    for entry in bench_registry::ALGORITHMS.iter() {
        if entry.name == name {
            return test_entry(entry);
        }
    }
    eprintln!("sort_test: '{}' not found in ALGORITHMS", name);
    false
}

pub fn test_all() -> bool {
    let mut all_ok = true;
    for entry in bench_registry::ALGORITHMS.iter() {
        if !test_entry(entry) {
            all_ok = false;
        }
    }
    all_ok
}

fn test_entry(entry: &AlgorithmEntry) -> bool {
    if entry.category != Category::Sort {
        // Non-sort categories have their own batteries inside `run_correctness`.
        (entry.run_correctness)();
        eprintln!("{}: OK", entry.name);
        return true;
    }
    // For sorts, the entry's run_correctness already covers a full
    // pattern bank via `sort_battery`. Defer to it so this helper stays
    // a single source of truth.
    (entry.run_correctness)();
    eprintln!("{}: OK", entry.name);
    true
}
