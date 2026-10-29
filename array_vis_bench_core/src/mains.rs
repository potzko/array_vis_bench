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

use crate::bench_registry::{entry_line, list_inputs, primary_input, AlgorithmEntry, Axes, Category, RunConfig, ALGORITHMS};
use crate::clock::{batch_size, time_runs};
use sort_logger::{SortLogger, VisualizerLogger};
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Duration;

/// One entry's timing over its rounds, under its name, category and axes: the median,
/// the inter quartile range as the spread, and how many rounds fell outside
/// the Tukey fences, one and a half ranges past the quartiles, as the outliers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timing {
    pub name: &'static str,
    pub category: Category,
    pub axes: Axes,
    pub median: Duration,
    pub iqr: Duration,
    pub outliers: usize,
}

/// The statistics of `entry`'s round times. The quartiles are the values at
/// a quarter and three quarters of the sorted rounds, so three rounds give the
/// whole spread as the range and never an outlier.
pub fn timing(entry: &AlgorithmEntry, runs: &mut [Duration]) -> Timing {
    runs.sort_unstable();
    let len = runs.len();
    let (q1, median, q3) = (runs[len / 4], runs[len / 2], runs[(3 * len / 4).min(len - 1)]);
    let iqr = q3 - q1;
    let fence = iqr + iqr / 2;
    let (low, high) = (q1.saturating_sub(fence), q3 + fence);
    let outliers = runs.iter().filter(|&&r| r < low || r > high).count();
    Timing { name: entry.name, category: entry.category, axes: entry.axes, median, iqr, outliers }
}

/// The outcome of running one consumer over a set of algorithms.
#[derive(Debug, Default)]
pub struct MainReport {
    pub consumer: &'static str,
    pub ran: usize,
    pub ok: usize,
    /// Names of the algorithms that failed (panicked, or produced no output).
    pub failures: Vec<String>,
    /// Each algorithm a timing consumer ran, in the order it ran them, with
    /// the statistics of its rounds. Empty for a consumer that does not time.
    pub timings: Vec<Timing>,
    /// The input size and run count the timings were taken at; `None` for a
    /// consumer that does not time.
    pub setup: Option<Benchmark>,
}

impl MainReport {
    pub fn all_ok(&self) -> bool {
        self.failures.is_empty()
    }

    /// The timings as a TSV archive: a `# n=N k=K input=I` header naming the
    /// setup, then one line per timed entry, the name, the median and the
    /// inter quartile range in nanoseconds and the outlier count, sorted by
    /// name so two archives diff cleanly.
    pub fn timings_tsv(&self) -> String {
        let header = self.setup.map(|b| format!("# n={} k={} input={}\n", b.n, b.k, b.input_label())).unwrap_or_default();
        let mut rows: Vec<&Timing> = self.timings.iter().collect();
        rows.sort_by_key(|t| t.name);
        let lines = rows.iter().map(|t| format!("{}\t{}\t{}\t{}\n", t.name, t.median.as_nanos(), t.iqr.as_nanos(), t.outliers));
        let entries = rows.iter().map(|t| entry_line(t.name, t.category, t.axes));
        header + &lines.chain(entries).collect::<String>()
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

/// Benchmark consumer: times each algorithm at size `n` over `k` rounds on
/// the category's primary input, or on `input` when one is named, the input
/// built before the clock starts and the run under the no op logger so only
/// the algorithm's work is measured; then prints one table per category of
/// the median run times, fastest first. `k` is at least 1, `run` clamps it.
///
/// An algorithm whose contract cap (`max_input_size`) is below `n` cannot run
/// at that size, and one whose category has no input of the named shape has
/// nothing to run on; both are skipped and counted, not failures. They are
/// excluded rather than clamped so every timed entry ran at the same `n` on
/// the same shape and the ranking is fair.
#[derive(Clone, Copy, Debug)]
pub struct Benchmark {
    pub n: usize,
    pub k: usize,
    pub input: Option<&'static str>,
}

impl Default for Benchmark {
    fn default() -> Self {
        // Several chaos sorts declare a cap of 1000, so none of them is skipped.
        Self { n: 1000, k: 3, input: None }
    }
}

impl Benchmark {
    /// Whether `entry`'s cap lets it run at this benchmark's `n` and its
    /// category has the input shape to run on.
    pub fn admits(&self, entry: &AlgorithmEntry) -> bool {
        entry.max_input_size.map_or(true, |cap| cap >= self.n) && self.input_for(entry).is_some()
    }

    /// The input `entry` runs on: the named shape when its category has it,
    /// else the category's primary input when no shape was named.
    pub fn input_for(&self, entry: &AlgorithmEntry) -> Option<&'static str> {
        match self.input {
            None => Some(primary_input(entry.category)),
            Some(name) => list_inputs(entry.category).into_iter().find(|&i| i == name),
        }
    }

    /// The name the archive header records for the input choice.
    pub fn input_label(&self) -> &'static str {
        self.input.unwrap_or("primary")
    }

    // One round of `entry`: a batch of `m` runs at `self.n` on seeds the round
    // and batch index pick, timed back to back, as the time of one run.
    fn one_round(&self, entry: &AlgorithmEntry, round: u64, m: usize) -> Option<Duration> {
        let input = self.input_for(entry)?;
        let seeds: Vec<u64> = (0..m as u64).map(|i| round * m as u64 + i).collect();
        time_runs(entry, input, self.n, &seeds)
    }
}

// One table per category, categories in name order, each table fastest first
// with the name column as wide as its longest name: the median, the inter
// quartile range and the outlier count of each entry's rounds.
fn tables(timed: &[(Category, Timing)]) -> String {
    let grouped = BTreeMap::<&str, Vec<Timing>>::new();
    let by_category = timed.iter().fold(grouped, |mut acc, &(category, t)| {
        acc.entry(category.as_str()).or_default().push(t);
        acc
    });
    by_category
        .into_iter()
        .map(|(category, mut rows)| {
            rows.sort_by_key(|t| t.median);
            let width = rows.iter().map(|t| t.name.len()).fold("algorithm".len(), usize::max);
            let header = [
                format!("=== {category} ({} variants) ===", rows.len()),
                format!("{:<width$}  {:>12}  {:>12}  {:>8}", "algorithm", "median", "iqr", "outliers"),
                format!("{:-<width$}  {:->12}  {:->12}  {:->8}", "", "", "", ""),
            ];
            let lines = rows.iter().map(|t| {
                format!("{:<width$}  {:>12.2?}  {:>12.2?}  {:>8}", t.name, t.median, t.iqr, t.outliers)
            });
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
        // One unmeasured run per entry first, so caches and branch predictors
        // have seen the code before a round is timed; its time also sizes the
        // batch a round times back to back, so a microsecond entry is measured
        // over many runs rather than one tick of the clock.
        let mut durations: Vec<Option<Vec<Duration>>> = vec![Some(Vec::with_capacity(k)); runnable.len()];
        let batches: Vec<usize> = runnable
            .iter()
            .zip(durations.iter_mut())
            .map(|(e, slot)| match self.one_round(e, u64::MAX / 2, 1) {
                Some(warm) => batch_size(warm, self.n),
                None => { *slot = None; 1 }
            })
            .collect();
        // Every entry runs once per round, so a burst of machine load lands on
        // different entries in different rounds and the per entry median
        // shrugs it off. Running one entry k times back to back would put a
        // whole burst into one median.
        for round in 0..k as u64 {
            for ((slot, e), &m) in durations.iter_mut().zip(&runnable).zip(&batches) {
                let Some(runs) = slot else { continue };
                match self.one_round(e, round, m) {
                    Some(d) => runs.push(d),
                    None => *slot = None,
                }
            }
        }
        let results: Vec<_> =
            runnable.iter().zip(durations).map(|(&e, runs)| (e, runs.map(|mut r| timing(e, &mut r)))).collect();
        let timed: Vec<(&AlgorithmEntry, Timing)> =
            results.iter().filter_map(|&(e, t)| t.map(|t| (e, t))).collect();
        let failed = results.iter().filter(|(_, t)| t.is_none());
        let r = MainReport {
            consumer: self.name(),
            ran: results.len(),
            ok: timed.len(),
            failures: failed.map(|(e, _)| e.name.to_string()).collect(),
            timings: timed.iter().map(|&(_, t)| t).collect(),
            setup: Some(*self),
        };
        println!(
            "=== benchmark: n={}, k={k}, input {}, {} timed, {} skipped (cap below n or no such input) ===",
            self.n, self.input_label(), r.ok, skipped.len()
        );
        let rows: Vec<_> = timed.iter().map(|&(e, t)| (e.category, t)).collect();
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
    fn record(arr: &mut [usize], _: usize, _: &mut dyn SortLogger<usize>) {
        RUNS.fetch_add(1, Ordering::SeqCst);
        SIZE.store(arr.len(), Ordering::SeqCst);
    }
    fn no_input_run(_: &str, _: &RunConfig, _: &mut dyn SortLogger<usize>) {}

    fn no_battery() {}

    static ORDER: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());
    fn record_first(_: &mut [usize], _: usize, _: &mut dyn SortLogger<usize>) {
        ORDER.lock().unwrap().push("first");
    }
    fn record_second(_: &mut [usize], _: usize, _: &mut dyn SortLogger<usize>) {
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
            run: record,
            run_with_input: no_input_run,
            run_correctness: no_battery,
        }
    }

    static FITS: AlgorithmEntry = entry("fits", Some(1000));
    static TOO_SMALL: AlgorithmEntry = entry("too small", Some(999));

    #[test]
    fn timings_tsv_names_the_setup_then_each_entry_by_name_in_nanoseconds_then_describes_each() {
        let zeta = Timing { axes: &[("Pivot", "first", "pivot"), ("SmallSort", "insertion: 32", "small")], ..flat("zeta", Duration::from_micros(3)) };
        let report = MainReport {
            timings: vec![zeta, flat("alpha", Duration::from_nanos(1500))],
            setup: Some(Benchmark { n: 1000, k: 15, input: None }),
            ..Default::default()
        };
        assert_eq!(
            report.timings_tsv(),
            "# n=1000 k=15 input=primary\nalpha\t1500\t0\t0\nzeta\t3000\t0\t0\n\
             # entry\talpha\tsort\n\
             # entry\tzeta\tsort\tPivot\tfirst\tpivot\tSmallSort\tinsertion: 32\tsmall\n"
        );
    }

    // A timing with no spread, as a table or archive test wants.
    fn flat(name: &'static str, median: Duration) -> Timing {
        Timing { name, category: Category::Sort, axes: &[], median, iqr: Duration::ZERO, outliers: 0 }
    }

    #[test]
    fn the_statistics_are_the_median_the_quartile_range_and_the_tukey_outliers() {
        let mut runs: Vec<Duration> = [4, 100, 2, 1, 3].map(ms).to_vec();
        let t = timing(&FITS, &mut runs);
        assert_eq!((t.median, t.iqr, t.outliers), (ms(3), ms(2), 1));
        let mut three: Vec<Duration> = [7, 5, 9].map(ms).to_vec();
        let t = timing(&FITS, &mut three);
        assert_eq!((t.median, t.iqr, t.outliers), (ms(7), ms(4), 0));
    }

    #[test]
    fn the_benchmark_runs_every_entry_once_per_round() {
        static FIRST: AlgorithmEntry = AlgorithmEntry { run: record_first, ..entry("first", None) };
        static SECOND: AlgorithmEntry = AlgorithmEntry { run: record_second, ..entry("second", None) };
        ORDER.lock().unwrap().clear();
        let report = Benchmark { n: 4, k: 3, input: None }.run(&[&FIRST, &SECOND]);
        // The warm up runs first, then the three rounds, each a batch of
        // `first` then a batch of `second`. A batch's size comes from the
        // warm up's time on the real clock, so it is read off the first round
        // rather than assumed.
        let order = ORDER.lock().unwrap();
        assert_eq!(&order[..2], ["first", "second"]);
        let rounds = &order[2..];
        let firsts = rounds.iter().take_while(|&&e| e == "first").count();
        let seconds = rounds[firsts..].iter().take_while(|&&e| e == "second").count();
        assert!(firsts >= 1 && seconds >= 1, "{rounds:?}");
        let round: Vec<&str> = std::iter::repeat_n("first", firsts).chain(std::iter::repeat_n("second", seconds)).collect();
        assert_eq!(rounds, round.repeat(3));
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
        let timed = [(Category::Sort, flat("slow", ms(5))), (Category::Sort, flat("fast", ms(1)))];
        let table = tables(&timed);
        let lines: Vec<&str> = table.lines().collect();
        assert_eq!(lines[0], "=== sort (2 variants) ===");
        assert!(lines[3].starts_with("fast") && lines[3].contains("1.00ms"), "{table}");
        assert!(lines[4].starts_with("slow") && lines[4].contains("5.00ms"), "{table}");
        assert!(lines[1].ends_with("iqr  outliers") && lines[4].ends_with("0"), "{table}");
        assert_eq!(lines[3].len(), lines[4].len(), "the median column lines up");
    }

    #[test]
    fn there_is_one_table_per_category_that_timed_something() {
        let timed = [
            (Category::Sort, flat("a sort", ms(2))),
            (Category::Rotation, flat("a rotation", ms(1))),
            (Category::Sort, flat("another sort", ms(3))),
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
        // One warm up run, then k rounds of a batch each; a run this short
        // gets the largest batch, but the test only relies on the round count.
        let runs = RUNS.load(Ordering::SeqCst);
        assert!(runs > 3 && (runs - 1) % 3 == 0, "a warm up then three rounds of one batch, got {runs}");
        assert_eq!(SIZE.load(Ordering::SeqCst), 1000);
        assert_eq!(report.timings.len(), 1);
        assert_eq!(report.timings[0].name, "fits");
    }
}
