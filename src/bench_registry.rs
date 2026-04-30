use linkme::distributed_slice;

pub struct SortBenchEntry {
    pub name: &'static str,
    pub big_o: &'static str,
    pub stable: bool,
    pub run: fn(&mut [usize]),
}

#[distributed_slice]
pub static BENCH_SORTS: [SortBenchEntry] = [..];

/// All registered bench entries in canonical menu order — depth-first
/// traversal of the registry's tree, which sorts each level by subtree
/// size so specialised (small-group) sorts surface first.
///
/// `linkme` makes no guarantee about link-time ordering, so consumers that
/// produce user-visible output should iterate this instead of `BENCH_SORTS`
/// directly. Bench output and UI menu therefore surface variants in the
/// same order without either side having to declare it.
pub fn sorted() -> Vec<&'static SortBenchEntry> {
    let order: std::collections::HashMap<String, usize> = sort_registry_core::get_registered_sorts()
        .into_iter()
        .enumerate()
        .map(|(i, n)| (n, i))
        .collect();
    let mut v: Vec<&'static SortBenchEntry> = BENCH_SORTS.iter().collect();
    v.sort_by_key(|e| (order.get(e.name).copied().unwrap_or(usize::MAX), e.name));
    v
}

pub fn for_each<F: FnMut(&'static SortBenchEntry)>(mut f: F) {
    for entry in sorted() {
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
    use crate::utils::array_gen::{get_arr, get_reversed_arr, get_rand_arr, get_rand_arr_in_range};
    use rand::Rng;
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    /// Run a sort on `arr` and verify the output is a sorted permutation of the input.
    fn run_and_verify(entry: &SortBenchEntry, arr: &mut Vec<usize>, label: &str) {
        let mut expected = arr.clone();
        expected.sort();
        (entry.run)(arr);
        assert_eq!(
            arr, &expected,
            "{}: failed on '{}' (n={})",
            entry.name, label, expected.len()
        );
    }

    /// Comprehensive correctness test for a single sort.
    pub fn check_sort(entry: &SortBenchEntry) {
        let mut rng = thread_rng();

        // ── Trivial cases ────────────────────────────────────────
        run_and_verify(entry, &mut vec![], "empty");
        run_and_verify(entry, &mut vec![1], "single");
        run_and_verify(entry, &mut vec![1, 2], "sorted pair");
        run_and_verify(entry, &mut vec![2, 1], "reversed pair");
        run_and_verify(entry, &mut vec![1, 1], "equal pair");

        // ── All permutations of small arrays ─────────────────────
        for n in 0..=5 {
            let base: Vec<usize> = (0..n).collect();
            for_each_permutation(&base, &mut |perm| {
                let mut arr = perm.to_vec();
                run_and_verify(entry, &mut arr, &format!("perm(n={n})"));
            });
        }

        // ── Structured patterns at several sizes ─────────────────
        for &n in &[16, 32, 33, 64, 128] {
            run_and_verify(entry, &mut get_arr(n), &format!("sorted {n}"));
            run_and_verify(entry, &mut get_reversed_arr(n), &format!("reversed {n}"));
            run_and_verify(entry, &mut vec![42; n], &format!("all-same {n}"));
        }

        // Alternating high-low
        run_and_verify(
            entry,
            &mut (0..128).map(|i| if i % 2 == 0 { i } else { 128 - i }).collect(),
            "alternating 128",
        );

        // Pipe organ: 0,1,2,...,n,...,2,1,0
        run_and_verify(
            entry,
            &mut (0..100usize).chain((0..99).rev()).collect(),
            "pipe organ 199",
        );

        // Sawtooth: repeating ascending runs
        run_and_verify(
            entry,
            &mut (0..200).map(|i| i % 20).collect(),
            "sawtooth 200",
        );

        // Nearly sorted: sorted with a few random swaps
        let mut nearly = get_arr(500);
        for _ in 0..10 {
            let a = rng.gen_range(0..500);
            let b = rng.gen_range(0..500);
            nearly.swap(a, b);
        }
        run_and_verify(entry, &mut nearly, "nearly sorted 500");

        // Sorted then reversed tail
        let mut sorted_rev_tail: Vec<usize> = (0..400).collect();
        sorted_rev_tail.extend((400..500).rev());
        run_and_verify(entry, &mut sorted_rev_tail, "sorted + reversed tail 500");

        // ── Duplicate-heavy patterns ─────────────────────────────
        run_and_verify(entry, &mut get_rand_arr_in_range(500, 0, 3), "few unique (3 vals, n=500)");
        run_and_verify(entry, &mut get_rand_arr_in_range(300, 0, 2), "binary (2 vals, n=300)");
        run_and_verify(entry, &mut get_rand_arr_in_range(500, 0, 50), "many dups (50 vals, n=500)");

        // ── Random cases ─────────────────────────────────────────
        for &n in &[100, 500, 1000] {
            // Random permutation (no duplicates)
            let mut perm = get_arr(n);
            perm.shuffle(&mut rng);
            run_and_verify(entry, &mut perm, &format!("random permutation {n}"));

            // Random values (with duplicates)
            run_and_verify(entry, &mut get_rand_arr_in_range(n, 0, n), &format!("random values {n}"));
        }

        // One larger random case
        let mut large = get_rand_arr(5000);
        run_and_verify(entry, &mut large, "random 5000");
    }

    /// Verify that a stable sort preserves relative order of equal elements.
    ///
    /// Encodes (value, original_index) pairs where comparison is by value only
    /// (value in high bits, index in low bits). After sorting, equal-valued
    /// elements must appear in ascending original-index order.
    pub fn check_sort_stable(entry: &SortBenchEntry) {
        if !entry.stable {
            return;
        }
        let value_bits = 32;
        let encode = |value: usize, index: usize| -> usize { (value << value_bits) | index };
        let decode_value = |x: usize| -> usize { x >> value_bits };
        let decode_index = |x: usize| -> usize { x & ((1 << value_bits) - 1) };

        let test_cases: Vec<(&str, Vec<usize>)> = vec![
            ("3 values, n=200", (0..200).map(|i| i % 3).collect()),
            ("2 values, n=100", (0..100).map(|i| i % 2).collect()),
            ("all equal, n=50", vec![7; 50]),
            ("10 values, n=500", get_rand_arr_in_range(500, 0, 10)),
        ];

        for (label, values) in &test_cases {
            let mut arr: Vec<usize> = values
                .iter()
                .enumerate()
                .map(|(i, &v)| encode(v, i))
                .collect();
            (entry.run)(&mut arr);

            for i in 1..arr.len() {
                assert!(
                    decode_value(arr[i - 1]) <= decode_value(arr[i]),
                    "{}: stability '{}' — not sorted at position {}",
                    entry.name, label, i
                );
                if decode_value(arr[i - 1]) == decode_value(arr[i]) {
                    assert!(
                        decode_index(arr[i - 1]) < decode_index(arr[i]),
                        "{}: stability '{}' — order violated at position {} \
                         (value={}, indices {} then {})",
                        entry.name, label, i,
                        decode_value(arr[i]),
                        decode_index(arr[i - 1]),
                        decode_index(arr[i]),
                    );
                }
            }
        }
    }

    /// Generate all permutations of `base` and call `f` on each (Heap's algorithm).
    fn for_each_permutation(base: &[usize], f: &mut dyn FnMut(&[usize])) {
        let mut arr = base.to_vec();
        let n = arr.len();
        heap_permute(&mut arr, n, f);
    }

    fn heap_permute(arr: &mut Vec<usize>, k: usize, f: &mut dyn FnMut(&[usize])) {
        if k <= 1 {
            f(arr);
            return;
        }
        heap_permute(arr, k - 1, f);
        for i in 0..k - 1 {
            if k % 2 == 0 {
                arr.swap(i, k - 1);
            } else {
                arr.swap(0, k - 1);
            }
            heap_permute(arr, k - 1, f);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run check_sort on every registered sort — catches sorts that are
    /// registered but somehow missing their own test module.
    #[test]
    fn all_registered_sorts_are_correct() {
        let mut count = 0;
        for entry in BENCH_SORTS.iter() {
            test_helpers::check_sort(entry);
            count += 1;
        }
        assert!(count > 0, "No sorts registered in BENCH_SORTS");
    }

    /// Verify stability for every sort that claims to be stable.
    #[test]
    fn all_stable_sorts_are_stable() {
        for entry in BENCH_SORTS.iter() {
            test_helpers::check_sort_stable(entry);
        }
    }
}
