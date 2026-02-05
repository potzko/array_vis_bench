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
