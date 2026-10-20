//! `Main` — the consumer abstraction.
//!
//! A `Main` is one of the pipelines a *set of registered algorithms* can be
//! driven through: visualise, correctness, benchmark. It is the Rust counterpart
//! of the AVBS query-language concept `consumers: List<Mains<Sorts>> =
//! [visualiser, correctness, benchmark]` — each consumer runs over the same
//! `&[&AlgorithmEntry]` (whatever `Sorts` resolved to) and reports a
//! [`MainReport`]. The algorithms themselves are registry-source-agnostic: an
//! entry emitted by the spec compiler and one registered by the legacy combo
//! path look identical here (both are just `AlgorithmEntry` rows).

use crate::bench_registry::{primary_input, AlgorithmEntry, Category, RunConfig, ALGORITHMS};
use sort_logger::{NoOpLogger, SortLogger, VisualizerLogger};
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

/// The outcome of running one consumer over a set of algorithms.
#[derive(Debug, Default)]
pub struct MainReport {
    pub consumer: &'static str,
    pub ran: usize,
    pub ok: usize,
    /// Names of the algorithms that failed (panicked, or produced no output).
    pub failures: Vec<String>,
    /// The median run time of each algorithm a timing consumer ran, in the
    /// order it ran them. Empty for a consumer that does not time.
    pub timings: Vec<(&'static str, Duration)>,
}

impl MainReport {
    pub fn all_ok(&self) -> bool {
        self.failures.is_empty()
    }

    /// The timings as a TSV archive, one `name<TAB>median nanoseconds` line per
    /// timed entry, sorted by name so two archives diff cleanly.
    pub fn timings_tsv(&self) -> String {
        let mut rows: Vec<_> = self.timings.iter().map(|(name, d)| (*name, d.as_nanos())).collect();
        rows.sort();
        rows.iter().map(|(name, ns)| format!("{name}\t{ns}\n")).collect()
    }
}

/// A consumer of a set of registered algorithms.
pub trait Main {
    /// The consumer's name (as it would appear in a `consumers: [...]` list).
    fn name(&self) -> &'static str;
    /// Drive every algorithm in `sorts` through this consumer's pipeline.
    fn run(&self, sorts: &[&'static AlgorithmEntry]) -> MainReport;
}

/// Run several consumers over the same set of algorithms — the direct analogue
/// of `consumers: List<Mains<Sorts>> = [visualiser, correctness, benchmark]`.
pub fn run_all(consumers: &[&dyn Main], sorts: &[&'static AlgorithmEntry]) -> Vec<MainReport> {
    consumers.iter().map(|c| c.run(sorts)).collect()
}

// ── the language bridge: name → Main impl, name → AlgorithmEntry rows ─────────

/// Map a consumer name (as written in an AVBS `consumers: [...]` list) to its
/// `Main` impl. `None` for an unknown name — the caller reports it. This is the
/// runtime half of the language's `Mains<…>` wrapper.
pub fn main_by_name(name: &str) -> Option<Box<dyn Main>> {
    match name {
        "visualiser" | "visualizer" => Some(Box::new(Visualiser::default())),
        "correctness" => Some(Box::new(Correctness)),
        "benchmark" => Some(Box::new(Benchmark::default())),
        _ => None,
    }
}

/// Select the registered `AlgorithmEntry` rows whose name is in `names` — the
/// runtime resolution of a `List<SortingAlgorithm>` set to its concrete entries.
/// (The solver produced those names as the labels of the family's ground sorts.)
/// A name with no matching entry is simply absent from the result; [`run_named`]
/// treats that as an error so a mismatch is never silent.
pub fn entries_for_names(names: &[&str]) -> Vec<&'static AlgorithmEntry> {
    ALGORITHMS.iter().filter(|e| names.contains(&e.name)).collect()
}

/// Run a consumer list over a named algorithm set — the whole `consumers:`
/// binding, executed. `main_names` are resolved via [`main_by_name`] and
/// `algo_names` via [`entries_for_names`]; the result is one [`MainReport`] per
/// consumer. Errors on an unknown consumer name OR an algorithm name with no
/// registered entry (a resolved-but-unregistered family — never silently dropped).
pub fn run_named(main_names: &[&str], algo_names: &[&str]) -> Result<Vec<MainReport>, String> {
    let consumers: Vec<Box<dyn Main>> = main_names
        .iter()
        .map(|n| main_by_name(n).ok_or_else(|| format!("unknown consumer `{n}`")))
        .collect::<Result<_, _>>()?;
    let entries = entries_for_names(algo_names);
    let present: std::collections::HashSet<&str> = entries.iter().map(|e| e.name).collect();
    let missing: Vec<&str> = algo_names.iter().copied().filter(|n| !present.contains(n)).collect();
    if !missing.is_empty() {
        return Err(format!(
            "no registered AlgorithmEntry for: {} (resolved by the query but never \
             linked/emitted)",
            missing.join(", ")
        ));
    }
    let refs: Vec<&dyn Main> = consumers.iter().map(|b| b.as_ref()).collect();
    Ok(run_all(&refs, &entries))
}

/// Correctness consumer: runs each algorithm's correctness battery, catching a
/// panic (a battery assertion failure) as a per-algorithm failure rather than
/// aborting the whole run.
pub struct Correctness;

impl Main for Correctness {
    fn name(&self) -> &'static str {
        "correctness"
    }
    fn run(&self, sorts: &[&'static AlgorithmEntry]) -> MainReport {
        let mut r = MainReport { consumer: self.name(), ..Default::default() };
        for e in sorts {
            r.ran += 1;
            match catch_unwind(AssertUnwindSafe(|| (e.run_correctness)())) {
                Ok(()) => r.ok += 1,
                Err(_) => r.failures.push(e.name.to_string()),
            }
        }
        r
    }
}

/// Visualiser consumer: drives each algorithm through a [`VisualizerLogger`] and
/// checks it produced a non-empty event log. Rendering the log to an MP4 is the
/// interactive picker's job (see [`crate::visualise`]); here "ticking" means the
/// algorithm ran and emitted observable operations.
pub struct Visualiser {
    pub config: RunConfig,
}

impl Default for Visualiser {
    fn default() -> Self {
        Self { config: RunConfig { size: 64, seed: 1 } }
    }
}

impl Main for Visualiser {
    fn name(&self) -> &'static str {
        "visualiser"
    }
    fn run(&self, sorts: &[&'static AlgorithmEntry]) -> MainReport {
        let mut r = MainReport { consumer: self.name(), ..Default::default() };
        for e in sorts {
            r.ran += 1;
            let input = primary_input(e.category);
            let mut logger =
                VisualizerLogger::<usize> { type_ghost: std::marker::PhantomData, log: Vec::new() };
            (e.run_with_input)(input, &self.config, &mut logger as &mut dyn SortLogger<usize>);
            if logger.log.is_empty() {
                r.failures.push(e.name.to_string());
            } else {
                r.ok += 1;
            }
        }
        r
    }
}

/// Benchmark consumer: times each algorithm on a random array of size `n`, `k`
/// times (fresh seed per run, [`NoOpLogger`] so only the algorithm's work is
/// measured), then prints one table per category of the median run times,
/// fastest first. `k` is at least 1, `run` clamps it.
///
/// An algorithm whose contract cap (`max_input_size`) is below `n` cannot run
/// at that size, so it is skipped and counted, not a failure. It is excluded
/// rather than clamped so every timed entry ran at the same `n` and the ranking
/// is fair.
#[derive(Clone, Copy, Debug)]
pub struct Benchmark {
    pub n: usize,
    pub k: usize,
}

impl Default for Benchmark {
    fn default() -> Self {
        // Several chaos sorts declare a cap of 1000, so none of them is skipped.
        Self { n: 1000, k: 3 }
    }
}

impl Benchmark {
    /// Whether `entry`'s cap lets it run at this benchmark's `n`.
    pub fn admits(&self, entry: &AlgorithmEntry) -> bool {
        entry.max_input_size.map_or(true, |cap| cap >= self.n)
    }

    // The median of `k` timed runs on seeds 0 to k - 1, `None` when one panicked.
    fn one_run(&self, entry: &AlgorithmEntry, seed: u64) -> Option<Duration> {
        let input = primary_input(entry.category);
        let config = RunConfig { size: self.n, seed };
        let start = Instant::now();
        catch_unwind(AssertUnwindSafe(|| (entry.run_with_input)(input, &config, &mut NoOpLogger)))
            .ok()
            .map(|()| start.elapsed())
    }
}

// One table per category, categories in name order, each table fastest first
// with the name column as wide as its longest name.
fn tables(timed: &[(Category, &str, Duration)]) -> String {
    let grouped = BTreeMap::<&str, Vec<_>>::new();
    let by_category = timed.iter().fold(grouped, |mut acc, &(category, name, median)| {
        acc.entry(category.as_str()).or_default().push((name, median));
        acc
    });
    by_category
        .into_iter()
        .map(|(category, mut rows)| {
            rows.sort_by_key(|&(_, median)| median);
            let width = rows.iter().map(|(name, _)| name.len()).fold("algorithm".len(), usize::max);
            let header = [
                format!("=== {category} ({} variants) ===", rows.len()),
                format!("{:<width$}  {:>12}", "algorithm", "median"),
                format!("{:-<width$}  {:->12}", "", ""),
            ];
            let lines =
                rows.iter().map(|(name, median)| format!("{name:<width$}  {median:>12.2?}"));
            header.into_iter().chain(lines).collect::<Vec<_>>().join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

impl Main for Benchmark {
    fn name(&self) -> &'static str {
        "benchmark"
    }
    fn run(&self, sorts: &[&'static AlgorithmEntry]) -> MainReport {
        let k = self.k.max(1);
        let (runnable, skipped): (Vec<_>, Vec<_>) =
            sorts.iter().copied().partition(|e| self.admits(e));
        // Every entry runs once per round, so a burst of machine load lands on
        // different entries in different rounds and the per entry median
        // shrugs it off. Running one entry k times back to back would put a
        // whole burst into one median.
        let mut durations: Vec<Option<Vec<Duration>>> = vec![Some(Vec::with_capacity(k)); runnable.len()];
        for round in 0..k as u64 {
            for (slot, e) in durations.iter_mut().zip(&runnable) {
                let Some(runs) = slot else { continue };
                match self.one_run(e, round) {
                    Some(d) => runs.push(d),
                    None => *slot = None,
                }
            }
        }
        let median = |runs: &mut Vec<Duration>| { runs.sort_unstable(); runs[runs.len() / 2] };
        let results: Vec<_> = runnable.iter().zip(durations).map(|(&e, runs)| (e, runs.map(|mut r| median(&mut r)))).collect();
        let timed: Vec<_> =
            results.iter().filter_map(|&(e, median)| median.map(|m| (e, m))).collect();
        let failed = results.iter().filter(|(_, median)| median.is_none());
        let r = MainReport {
            consumer: self.name(),
            ran: results.len(),
            ok: timed.len(),
            failures: failed.map(|(e, _)| e.name.to_string()).collect(),
            timings: timed.iter().map(|&(e, median)| (e.name, median)).collect(),
        };
        println!(
            "=== benchmark: n={}, k={k}, {} timed, {} skipped (cap < n) ===",
            self.n, r.ok, skipped.len()
        );
        let rows: Vec<_> = timed.iter().map(|&(e, median)| (e.category, e.name, median)).collect();
        println!("\n{}", tables(&rows));
        if !r.failures.is_empty() {
            println!("{} panicked: {}", r.failures.len(), r.failures.join(", "));
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use array_vis_bench_traits::Complexity;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static RUNS: AtomicUsize = AtomicUsize::new(0);
    static SIZE: AtomicUsize = AtomicUsize::new(0);

    // Records how often and at what size it ran, so a test can read the
    // consumer's `k` and `n` back out of the entry it drove.
    fn record(_input: &str, config: &RunConfig, _logger: &mut dyn SortLogger<usize>) {
        RUNS.fetch_add(1, Ordering::SeqCst);
        SIZE.store(config.size, Ordering::SeqCst);
    }

    fn no_battery() {}

    static ORDER: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());
    fn record_first(_input: &str, _config: &RunConfig, _logger: &mut dyn SortLogger<usize>) {
        ORDER.lock().unwrap().push("first");
    }
    fn record_second(_input: &str, _config: &RunConfig, _logger: &mut dyn SortLogger<usize>) {
        ORDER.lock().unwrap().push("second");
    }

    const fn entry(name: &'static str, cap: Option<usize>) -> AlgorithmEntry {
        AlgorithmEntry {
            name,
            category: Category::Sort,
            worst: Complexity::N1,
            best: Complexity::N1,
            average: Complexity::N1,
            space: Complexity::CONST,
            stable: true,
            adaptive: false,
            nondeterministic: false,
            max_input_size: cap,
            menu: &[],
            axes: &[],
            label_template: "",
            run_with_input: record,
            run_correctness: no_battery,
        }
    }

    static FITS: AlgorithmEntry = entry("fits", Some(1000));
    static TOO_SMALL: AlgorithmEntry = entry("too small", Some(999));

    #[test]
    fn timings_tsv_lists_each_entry_by_name_in_nanoseconds() {
        let report = MainReport {
            timings: vec![("zeta", Duration::from_micros(3)), ("alpha", Duration::from_nanos(1500))],
            ..Default::default()
        };
        assert_eq!(report.timings_tsv(), "alpha\t1500\nzeta\t3000\n");
    }

    #[test]
    fn the_benchmark_runs_every_entry_once_per_round() {
        static FIRST: AlgorithmEntry = AlgorithmEntry { run_with_input: record_first, ..entry("first", None) };
        static SECOND: AlgorithmEntry = AlgorithmEntry { run_with_input: record_second, ..entry("second", None) };
        ORDER.lock().unwrap().clear();
        let report = Benchmark { n: 4, k: 3 }.run(&[&FIRST, &SECOND]);
        assert_eq!(*ORDER.lock().unwrap(), ["first", "second", "first", "second", "first", "second"]);
        assert_eq!((report.ran, report.ok, report.timings.len()), (2, 2, 2));
    }

    #[test]
    fn benchmark_defaults_to_1000_and_3() {
        let defaults = Benchmark::default();
        assert_eq!((defaults.n, defaults.k), (1000, 3));
    }

    fn ms(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    #[test]
    fn a_table_lists_the_fastest_first_with_right_aligned_medians() {
        let timed = [(Category::Sort, "slow", ms(5)), (Category::Sort, "fast", ms(1))];
        let table = tables(&timed);
        let lines: Vec<&str> = table.lines().collect();
        assert_eq!(lines[0], "=== sort (2 variants) ===");
        assert!(lines[3].starts_with("fast") && lines[3].ends_with("1.00ms"), "{table}");
        assert!(lines[4].starts_with("slow") && lines[4].ends_with("5.00ms"), "{table}");
        assert_eq!(lines[3].len(), lines[4].len(), "the median column lines up");
    }

    #[test]
    fn there_is_one_table_per_category_that_timed_something() {
        let timed = [
            (Category::Sort, "a sort", ms(2)),
            (Category::Rotation, "a rotation", ms(1)),
            (Category::Sort, "another sort", ms(3)),
        ];
        let table = tables(&timed);
        let headers: Vec<&str> = table.lines().filter(|l| l.starts_with("===")).collect();
        assert_eq!(headers, ["=== rotation (1 variants) ===", "=== sort (2 variants) ==="]);
        assert!(tables(&[]).is_empty(), "no entries, no tables");
    }

    #[test]
    fn the_benchmark_by_name_runs_at_the_defaults() {
        let benchmark = main_by_name("benchmark").expect("benchmark is registered by name");
        assert_eq!(benchmark.name(), "benchmark");
        let report = benchmark.run(&[&FITS, &TOO_SMALL]);
        assert_eq!((report.ran, report.ok), (1, 1), "the entry capped below n is skipped");
        assert_eq!(RUNS.load(Ordering::SeqCst), 3, "k runs of the one entry that ran");
        assert_eq!(SIZE.load(Ordering::SeqCst), 1000);
        assert_eq!(report.timings.len(), 1);
        assert_eq!(report.timings[0].0, "fits");
    }
}
