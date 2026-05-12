use linkme::distributed_slice;

pub struct SortBenchEntry {
    pub name: &'static str,
    pub big_o: &'static str,
    pub stable: bool,
    pub run: fn(&mut [usize]),
}

#[distributed_slice]
pub static BENCH_SORTS: [SortBenchEntry] = [..];

/// Opt-in cap registry: `(sort_name, max_n_for_random_inputs)` pairs.
/// Sorts that can't handle large random inputs in reasonable time add
/// themselves via `register_test_cap!` (or a manual `distributed_slice`
/// entry); `max_n_for_tests` looks the cap up by name. Default is
/// "no cap" — no change needed at the call site for fast sorts.
#[distributed_slice]
pub static SORT_TEST_CAPS: [(&'static str, usize)] = [..];

/// Return the random-input size cap declared for `sort_name`, if any.
/// Used by `correctness::check_sort` to skip oversized random arrays
/// for slow sorts.
pub fn max_n_for_tests(sort_name: &str) -> Option<usize> {
    SORT_TEST_CAPS
        .iter()
        .find(|(name, _)| *name == sort_name)
        .map(|(_, cap)| *cap)
}

/// Declare a random-input size cap for a sort. Place near the sort's
/// `sort_family!` invocation:
///
/// ```text
/// register_test_cap!("bad heap sort", 1000);
/// ```
#[macro_export]
macro_rules! register_test_cap {
    ($name:expr, $cap:expr) => {
        const _: () = {
            #[::linkme::distributed_slice($crate::bench_registry::SORT_TEST_CAPS)]
            #[allow(non_upper_case_globals)]
            static CAP: (&'static str, usize) = ($name, $cap);
        };
    };
}

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

/// Environment variable name used to put a subprocess into
/// "run check_sort and exit" mode (see [`subprocess_dispatch`]).
const SUBPROCESS_ENV_VAR: &str = "AVB_RUN_CHECK_SORT";

/// Ctor-style early-init that hijacks the process before libtest's main
/// runs. When the parent sets `AVB_RUN_CHECK_SORT=<sort name>` and
/// re-execs the same binary, the child enters here, looks the sort up
/// in `BENCH_SORTS`, runs `check_sort`, and exits — never reaching the
/// test runner at all. The result: the subprocess always shares the
/// parent's exact build, so a freshly-added sort is immediately
/// available without rebuilding a separate runner binary.
#[ctor::ctor]
fn subprocess_dispatch() {
    let Ok(sort_name) = std::env::var(SUBPROCESS_ENV_VAR) else { return };
    let entry = BENCH_SORTS
        .iter()
        .find(|e| e.name == sort_name)
        .unwrap_or_else(|| {
            eprintln!("sort not registered: {sort_name}");
            std::process::exit(2);
        });
    correctness::check_sort(entry);
    std::process::exit(0);
}

/// Correctness-test runner. Public so the subprocess dispatch above can
/// call it; not `#[cfg(test)]` for the same reason.
pub mod correctness {
    use super::SortBenchEntry;
    use crate::utils::array_gen::{get_arr, get_reversed_arr, get_rand_arr, get_rand_arr_in_range};
    use rand::Rng;
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    /// Run a sort on `arr` and verify the result is a sorted permutation
    /// of the input. Panics on mismatch. Emits a `RUNNING: <label>` line
    /// to stderr before each call so that — when the subprocess is killed
    /// by an outer timeout — the parent can drain stderr and recover the
    /// exact input that was in flight.
    fn run_and_verify(entry: &SortBenchEntry, arr: &mut Vec<usize>, label: &str) {
        use std::io::Write;
        let _ = writeln!(
            std::io::stderr(),
            "RUNNING: '{}' (n={})",
            label,
            arr.len()
        );
        let _ = std::io::stderr().flush();
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
    ///
    /// Random-input sizes are capped at `entry.max_n_for_tests` (default
    /// `usize::MAX`); structured patterns always run. Runs synchronously
    /// in-process — wall-clock timeout / killing on TLE is handled by the
    /// subprocess wrapper that calls this.
    pub fn check_sort(entry: &SortBenchEntry) {
        let mut rng = thread_rng();
        let cap = super::max_n_for_tests(entry.name).unwrap_or(usize::MAX);

        // Skip inputs larger than the sort's declared cap (applies to
        // both random and structured patterns — for a sort that can't
        // handle n=500, that's true regardless of input shape). Small
        // trivial cases (empty, single, pairs, perms of n<=5) always
        // run.
        macro_rules! check {
            ($arr:expr, $label:expr) => {{
                let arr_vec: Vec<usize> = $arr.into_iter().collect();
                if arr_vec.len() <= cap {
                    run_and_verify(entry, &mut { arr_vec }, $label)
                }
            }};
        }
        // Alias kept for readability at the random-input call sites.
        macro_rules! check_rand {
            ($n:expr, $arr:expr, $label:expr) => {
                check!($arr, $label);
            };
        }

        // ── Trivial cases ────────────────────────────────────────
        check!(vec![], "empty");
        check!(vec![1], "single");
        check!(vec![1, 2], "sorted pair");
        check!(vec![2, 1], "reversed pair");
        check!(vec![1, 1], "equal pair");

        // ── All permutations of small arrays ─────────────────────
        for n in 0..=5 {
            let base: Vec<usize> = (0..n).collect();
            let mut perms: Vec<Vec<usize>> = Vec::new();
            for_each_permutation(&base, &mut |perm| perms.push(perm.to_vec()));
            for perm in perms {
                check!(perm, &format!("perm(n={n})"));
            }
        }

        // ── Structured patterns at several sizes ─────────────────
        for &n in &[16usize, 32, 33, 64, 128] {
            check!(get_arr(n), &format!("sorted {n}"));
            check!(get_reversed_arr(n), &format!("reversed {n}"));
            check!(vec![42usize; n], &format!("all-same {n}"));
        }

        check!(
            (0..128usize).map(|i| if i % 2 == 0 { i } else { 128 - i }).collect::<Vec<_>>(),
            "alternating 128"
        );
        check!(
            (0..100usize).chain((0..99).rev()).collect::<Vec<_>>(),
            "pipe organ 199"
        );
        check!(
            (0..200usize).map(|i| i % 20).collect::<Vec<_>>(),
            "sawtooth 200"
        );

        // Nearly sorted: sorted with a few random swaps
        let mut nearly = get_arr(500);
        for _ in 0..10 {
            let a = rng.gen_range(0..500);
            let b = rng.gen_range(0..500);
            nearly.swap(a, b);
        }
        check_rand!(500, nearly, "nearly sorted 500");

        // Sorted then reversed tail (deterministic, not capped)
        let mut sorted_rev_tail: Vec<usize> = (0..400).collect();
        sorted_rev_tail.extend((400..500).rev());
        check!(sorted_rev_tail, "sorted + reversed tail 500");

        // ── Duplicate-heavy patterns (random within range) ───────
        check_rand!(500, get_rand_arr_in_range(500, 0, 3), "few unique (3 vals, n=500)");
        check_rand!(300, get_rand_arr_in_range(300, 0, 2), "binary (2 vals, n=300)");
        check_rand!(500, get_rand_arr_in_range(500, 0, 50), "many dups (50 vals, n=500)");

        // ── Random cases ─────────────────────────────────────────
        for &n in &[100usize, 500, 1000] {
            let mut perm = get_arr(n);
            perm.shuffle(&mut rng);
            check_rand!(n, perm, &format!("random permutation {n}"));

            check_rand!(
                n,
                get_rand_arr_in_range(n, 0, n),
                &format!("random values {n}")
            );
        }

        check_rand!(5000, get_rand_arr(5000), "random 5000");
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
pub(crate) mod test_helpers {
    use super::{SortBenchEntry, SUBPROCESS_ENV_VAR};
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    /// Default wall-clock budget per sort. With `max_n_for_tests` caps in
    /// place, this is purely a safety net for genuinely-hung sorts —
    /// well-behaved sorts finish in well under a second.
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);

    /// Global serialisation lock for subprocess spawns. Cargo test runs
    /// `#[test]` functions on a thread pool by default, so without this
    /// every per-sort test plus the aggregate test would each spawn a
    /// child concurrently — defeating the "one-at-a-time, name printed,
    /// killed if stuck" debugging experience. The mutex makes subprocess
    /// spawn-to-completion atomic across all tests in the same
    /// `cargo test` process.
    static SUBPROCESS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Run `check_sort` for one entry in an isolated subprocess with a
    /// wall-clock timeout. The subprocess is just the current test
    /// binary re-executed with `AVB_RUN_CHECK_SORT=<sort name>`; the
    /// ctor in `bench_registry.rs` intercepts that env var, runs
    /// `check_sort`, and exits before libtest's main can touch
    /// anything. This means the subprocess always carries the same
    /// `BENCH_SORTS` linkme slice as the parent — no stale-binary risk.
    ///
    /// On TLE the subprocess is killed (real OS-level kill, no leaked
    /// threads). Returns `Ok(())` on success or `Err(message)` on TLE /
    /// non-zero exit / spawn failure; callers decide whether to panic
    /// (per-sort tests) or accumulate failures (the aggregate test).
    pub fn check_sort_subprocess(
        entry: &SortBenchEntry,
        timeout: Duration,
    ) -> Result<(), String> {
        // Serialise across all tests in this process so output stays
        // readable and only one subprocess runs at a time.
        let _guard = SUBPROCESS_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let exe = std::env::current_exe()
            .map_err(|e| format!("current_exe failed: {e}"))?;
        let mut child = Command::new(&exe)
            .env(SUBPROCESS_ENV_VAR, entry.name)
            // Discard any libtest args that might be on the parent's
            // argv — the ctor doesn't care and libtest never runs.
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("spawn {} failed: {}", exe.display(), e))?;

        let start = Instant::now();
        loop {
            match child.try_wait().expect("try_wait") {
                Some(status) => {
                    if status.success() {
                        return Ok(());
                    }
                    let output = child.wait_with_output().expect("wait_with_output");
                    return Err(format!(
                        "{}: check_sort failed (exit {:?}).\nstderr:\n{}",
                        entry.name,
                        status.code(),
                        String::from_utf8_lossy(&output.stderr).trim()
                    ));
                }
                None => {
                    if start.elapsed() >= timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        // Drain whatever the subprocess wrote to stderr
                        // before we killed it; the last `RUNNING:` line
                        // identifies the input that was in flight.
                        let mut buf = String::new();
                        if let Some(mut s) = child.stderr.take() {
                            use std::io::Read;
                            let _ = s.read_to_string(&mut buf);
                        }
                        let in_flight = buf
                            .lines()
                            .rev()
                            .find(|l| l.starts_with("RUNNING:"))
                            .map(|l| l.trim_start_matches("RUNNING:").trim())
                            .unwrap_or("<no input started>");
                        return Err(format!(
                            "{}: TLE on {} (timeout {:?}). If this sort is \
                             genuinely slow on large inputs, set \
                             `max_n_for_tests = N` in its `sort_family!` \
                             invocation to skip oversized random arrays.",
                            entry.name, in_flight, timeout
                        ));
                    }
                    thread::sleep(Duration::from_millis(5));
                }
            }
        }
    }

    /// Panic-on-failure wrapper around `check_sort_subprocess`. Used by
    /// the per-sort `__sf_*_test::correctness` tests where each test is
    /// independent and a TLE means that one test fails (cleanly, without
    /// blocking the rest of the run).
    pub fn check_sort_subprocess_assert(entry: &SortBenchEntry, timeout: Duration) {
        if let Err(msg) = check_sort_subprocess(entry, timeout) {
            panic!("{msg}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_helpers::{check_sort_subprocess, DEFAULT_TIMEOUT};

    /// Run check_sort (in subprocess) on every registered sort. Each
    /// sort prints `[i/N] sort name … ok|FAIL|TLE` on its own line so
    /// progress is visible in real time; failures accumulate into a
    /// summary panic at the end so one bad sort doesn't mask others.
    #[test]
    fn all_registered_sorts_are_correct() {
        use std::io::Write;
        let total = BENCH_SORTS.len();
        let mut failures: Vec<(String, String)> = Vec::new(); // (sort name, error message)
        let mut stderr = std::io::stderr();
        let suite_start = std::time::Instant::now();
        for (idx, entry) in BENCH_SORTS.iter().enumerate() {
            let prefix = format!("[{}/{}] {}", idx + 1, total, entry.name);
            let _ = write!(stderr, "  {prefix} ... ");
            let _ = stderr.flush();
            let start = std::time::Instant::now();
            match check_sort_subprocess(entry, DEFAULT_TIMEOUT) {
                Ok(()) => {
                    let _ = writeln!(stderr, "ok ({:?})", start.elapsed());
                }
                Err(msg) => {
                    let _ = writeln!(stderr, "FAIL");
                    let _ = writeln!(stderr, "    {msg}");
                    failures.push((entry.name.to_string(), msg));
                }
            }
        }
        assert!(total > 0, "No sorts registered in BENCH_SORTS");

        let elapsed = suite_start.elapsed();
        let _ = writeln!(
            stderr,
            "\nSUMMARY: {} ok, {} failed of {} total ({:?})",
            total - failures.len(),
            failures.len(),
            total,
            elapsed
        );
        if !failures.is_empty() {
            let _ = writeln!(stderr, "Failed sorts:");
            for (name, _) in &failures {
                let _ = writeln!(stderr, "  ✘ {name}");
            }
            panic!(
                "{} of {} sorts failed correctness check:\n\n{}",
                failures.len(),
                total,
                failures
                    .into_iter()
                    .map(|(_, m)| m)
                    .collect::<Vec<_>>()
                    .join("\n\n")
            );
        }
    }

    /// Verify stability for every sort that claims to be stable. Runs
    /// in-process (no subprocess) — stability tests use only small,
    /// fast inputs.
    #[test]
    fn all_stable_sorts_are_stable() {
        for entry in BENCH_SORTS.iter() {
            super::correctness::check_sort_stable(entry);
        }
    }
}
