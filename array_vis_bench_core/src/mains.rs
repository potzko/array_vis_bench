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

use crate::bench_registry::{primary_input, AlgorithmEntry, RunConfig, ALGORITHMS};
use sort_logger::{NoOpLogger, SortLogger, VisualizerLogger};
use std::panic::{catch_unwind, AssertUnwindSafe};

/// The outcome of running one consumer over a set of algorithms.
#[derive(Debug, Default)]
pub struct MainReport {
    pub consumer: &'static str,
    pub ran: usize,
    pub ok: usize,
    /// Names of the algorithms that failed (panicked, or produced no output).
    pub failures: Vec<String>,
}

impl MainReport {
    pub fn all_ok(&self) -> bool {
        self.failures.is_empty()
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

/// Benchmark consumer: runs each algorithm with a [`NoOpLogger`] (so only the
/// algorithm's work is exercised; timing is left to the caller). A panic is a
/// failure. Included for parity with the AVBS consumer list.
pub struct Benchmark {
    pub config: RunConfig,
}

impl Default for Benchmark {
    fn default() -> Self {
        Self { config: RunConfig { size: 256, seed: 0 } }
    }
}

impl Main for Benchmark {
    fn name(&self) -> &'static str {
        "benchmark"
    }
    fn run(&self, sorts: &[&'static AlgorithmEntry]) -> MainReport {
        let mut r = MainReport { consumer: self.name(), ..Default::default() };
        for e in sorts {
            r.ran += 1;
            let input = primary_input(e.category);
            let res = catch_unwind(AssertUnwindSafe(|| {
                (e.run_with_input)(input, &self.config, &mut NoOpLogger);
            }));
            match res {
                Ok(()) => r.ok += 1,
                Err(_) => r.failures.push(e.name.to_string()),
            }
        }
        r
    }
}
