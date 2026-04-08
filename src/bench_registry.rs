use linkme::distributed_slice;

pub struct SortBenchEntry {
    pub name: &'static str,
    pub big_o: &'static str,
    pub stable: bool,
    pub run: fn(&mut [usize]),
}

#[distributed_slice]
pub static BENCH_SORTS: [SortBenchEntry] = [..];

pub fn for_each<F: FnMut(&'static SortBenchEntry)>(mut f: F) {
    for entry in BENCH_SORTS {
        f(entry);
    }
}

#[macro_export]
macro_rules! for_each_bench_sort {
    ($entry:ident, $body:block) => {
        for $entry in $crate::bench_registry::BENCH_SORTS {
            $body
        }
    };
}

#[cfg(test)]
pub(crate) mod test_helpers {
    use super::SortBenchEntry;

    pub fn check_sort(entry: &SortBenchEntry) {
        let cases: Vec<Vec<usize>> = vec![
            vec![],
            vec![1],
            vec![2, 1],
            vec![1, 2],
            (0..32).rev().collect(),
            (0..32).collect(),
            vec![5; 32],
            (0..33).map(|i| if i % 2 == 0 { i } else { 33 - i }).collect(),
            (0..100).rev().collect(),
        ];
        for case in &cases {
            let mut arr = case.clone();
            let mut expected = case.clone();
            expected.sort();
            (entry.run)(&mut arr);
            assert_eq!(arr, expected, "{}: sort failed", entry.name);
        }
    }
}
