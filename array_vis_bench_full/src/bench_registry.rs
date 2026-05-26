//! Re-export of `array_vis_bench_core::bench_registry` plus the
//! test-only subprocess harness, which has to live in the wiring crate
//! so it sees the full populated `ALGORITHMS` slice (algorithm leaves
//! link into it; running this from `array_vis_bench_core` standalone
//! would see an empty registry).

pub use array_vis_bench_core::bench_registry::*;

#[cfg(test)]
pub(crate) mod test_helpers {
    use super::{AlgorithmEntry, SUBPROCESS_ENV_VAR};
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    /// Default wall-clock budget per algorithm. With `max_n_for_tests`
    /// caps in place, this is purely a safety net for genuinely-hung
    /// algorithms — well-behaved ones finish in well under a second.
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);

    /// Global serialisation lock for subprocess spawns. Cargo test runs
    /// `#[test]` functions on a thread pool by default, so without this
    /// every per-algorithm test plus the aggregate test would each spawn
    /// a child concurrently — defeating the "one-at-a-time, name
    /// printed, killed if stuck" debugging experience. The mutex makes
    /// subprocess spawn-to-completion atomic across all tests in the
    /// same `cargo test` process.
    static SUBPROCESS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Run the category battery for one entry in an isolated subprocess
    /// with a wall-clock timeout. The subprocess is just the current
    /// test binary re-executed with `AVB_RUN_CHECK_SORT=<algorithm name>`;
    /// the ctor in `bench_registry.rs` intercepts that env var, runs the
    /// entry's `run_correctness`, and exits before libtest's main can
    /// touch anything. The subprocess always carries the same
    /// `ALGORITHMS` linkme slice as the parent — no stale-binary risk.
    ///
    /// On TLE the subprocess is killed. Returns `Ok(())` on success or
    /// `Err(message)` on TLE / non-zero exit / spawn failure.
    pub fn check_sort_subprocess(
        entry: &AlgorithmEntry,
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
                        "{}: correctness failed (exit {:?}).\nstderr:\n{}",
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
                             `max_n_for_tests = N` in its `family!` \
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
    /// the per-algorithm `__sf_*_test::correctness` tests where each
    /// test is independent and a TLE means that one test fails
    /// (cleanly, without blocking the rest of the run).
    pub fn check_sort_subprocess_assert(entry: &AlgorithmEntry, timeout: Duration) {
        if let Err(msg) = check_sort_subprocess(entry, timeout) {
            panic!("{msg}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_helpers::{check_sort_subprocess, DEFAULT_TIMEOUT};

    /// Run the category battery (in subprocess) for every registered
    /// algorithm. Each prints `[i/N] name … ok|FAIL|TLE` so progress is
    /// visible in real time; failures accumulate into a summary panic at
    /// the end so one bad entry doesn't mask others.
    #[test]
    fn all_registered_algorithms_are_correct() {
        use std::io::Write;
        let total = ALGORITHMS.len();
        let mut failures: Vec<(String, String)> = Vec::new();
        let mut stderr = std::io::stderr();
        let suite_start = std::time::Instant::now();
        for (idx, entry) in ALGORITHMS.iter().enumerate() {
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
        assert!(total > 0, "No algorithms registered in ALGORITHMS");

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
            let _ = writeln!(stderr, "Failed:");
            for (name, _) in &failures {
                let _ = writeln!(stderr, "  ✘ {name}");
            }
            panic!(
                "{} of {} algorithms failed correctness check:\n\n{}",
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
}
