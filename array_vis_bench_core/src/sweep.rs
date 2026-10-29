//! The scaling sweep: how far each entry gets within a time budget. `sizes`
//! is the size ladder, `sweep` runs every entry up the ladder until one run
//! exceeds its share of the budget, then spends what is left binary searching
//! each entry's largest size that fits the share, and `Sweep` is the runner
//! that times real entries. The budget pays for everything a measurement
//! costs, the inputs built included, not only the timed runs, and no climb
//! passes the sweep's size ceiling, so an entry whose run barely grows with
//! the size cannot climb until its inputs exhaust memory.
//! `SweepReport::table` and `archive` print it.

use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, Instant};

use crate::bench_registry::{list_inputs, primary_input, AlgorithmEntry};
use crate::clock::{batch_size, time_runs};

/// The size ladder: every size to 100, then by tens to 200, by hundreds to
/// 1000, then powers of two. Dense where sorts are cheap and sparse where one
/// run already costs seconds.
pub fn sizes() -> impl Iterator<Item = usize> {
    let tens = (110..=200).step_by(10);
    let hundreds = (300..=1000).step_by(100);
    let powers = (11..usize::BITS).map(|exp| 1usize << exp);
    (1..=100).chain(tens).chain(hundreds).chain(powers)
}

/// The size ceiling a sweep climbs to unless told otherwise: 4 Mi elements,
/// 32 MiB of `usize` per input. Far past where a sort fits any sensible
/// share, so it is met only by entries whose run does not grow with the size.
pub const MAX_SIZE: usize = 1 << 22;

/// Why an entry's climb up the ladder ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stop {
    /// A run at this size took more than the share.
    Over(usize),
    /// The entry's contract cap, the last size it may take.
    Cap(usize),
    /// The sweep's size ceiling, below the entry's cap.
    Ceiling(usize),
    /// A run at this size panicked.
    Failed(usize),
}

/// One entry's climb: every timed size, the search's runs included, in size
/// order, and why the climb stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scaling {
    pub name: &'static str,
    pub points: Vec<(usize, Duration)>,
    pub stop: Stop,
    /// The largest size known to fit the share once the search has run: the
    /// last ladder size that fit, moved up by the binary search.
    pub fits: usize,
}

/// One measurement at a size: the time of one run, which the limit judges,
/// and what taking it cost, the inputs built and every run of the batch,
/// which the budget pays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Measure {
    pub run: Duration,
    pub cost: Duration,
}

/// The sweep's outcome: the share one run may take, what the measurements
/// cost in all, and every entry's climb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SweepReport {
    pub budget: Duration,
    pub spent: Duration,
    pub share: u32,
    pub input: &'static str,
    pub max_size: usize,
    pub limit: Duration,
    pub scalings: Vec<Scaling>,
}

/// Run the sweep `plan` over `entries` with `time` as the clock:
/// `time(entry, n)` measures one run at size `n`, `None` when it panicked.
/// One run may take at most `budget / (entries * share)`, the limit. Every
/// entry climbs the ladder in turn until a run exceeds the limit, its cap or
/// the ceiling stops it or a run fails; then
/// the time the climbs left over goes round robin, one run per entry per
/// round, to binary search between the last size that fit and the first that
/// did not, until every search converges or the budget is spent. A failed
/// run costs the limit.
pub fn sweep(
    entries: &[&'static AlgorithmEntry],
    plan: &Sweep,
    time: &mut dyn FnMut(&AlgorithmEntry, usize) -> Option<Measure>,
) -> SweepReport {
    let Sweep { budget, share, input, max_size } = *plan;
    let runs = (entries.len() as u32 * share).max(1);
    let limit = budget / runs;
    let mut spent = Duration::ZERO;
    let mut scalings: Vec<Scaling> =
        entries.iter().map(|e| climb(e, limit, max_size, time, &mut spent)).collect();

    let mut searching: Vec<(usize, usize, usize)> = scalings
        .iter()
        .enumerate()
        .filter_map(|(ind, s)| match s.stop { Stop::Over(over) => Some((ind, s.fits, over)), _ => None })
        .collect();
    while spent < budget && !searching.is_empty() {
        for (ind, lo, hi) in &mut searching {
            if spent >= budget { break; }
            let mid = (*lo + *hi) / 2;
            match time(entries[*ind], mid) {
                Some(m) => {
                    if m.run > limit { *hi = mid } else { *lo = mid }
                    spent += m.cost;
                    scalings[*ind].points.push((mid, m.run));
                }
                None => { *hi = mid; spent += limit; }
            }
        }
        for &(ind, lo, _) in &searching { scalings[ind].fits = lo; }
        searching.retain(|&(_, lo, hi)| hi - lo > 1);
    }
    for s in &mut scalings { s.points.sort_by_key(|&(n, _)| n); }
    SweepReport { budget, spent, share, input: input.unwrap_or("primary"), max_size, limit, scalings }
}

// One entry up the ladder until a run exceeds `limit`, the cap or the
// ceiling, or a run fails.
fn climb(
    entry: &'static AlgorithmEntry,
    limit: Duration,
    max_size: usize,
    time: &mut dyn FnMut(&AlgorithmEntry, usize) -> Option<Measure>,
    spent: &mut Duration,
) -> Scaling {
    let mut points = Vec::new();
    let mut fits = 0;
    let cap = entry.max_input_size.unwrap_or(usize::MAX);
    let mut stop = if cap <= max_size { Stop::Cap(cap) } else { Stop::Ceiling(max_size) };
    for n in sizes().take_while(|&n| n <= cap.min(max_size)) {
        match time(entry, n) {
            None => { *spent += limit; stop = Stop::Failed(n); break; }
            Some(m) => {
                *spent += m.cost;
                points.push((n, m.run));
                if m.run > limit { stop = Stop::Over(n); break; }
                fits = n;
            }
        }
    }
    Scaling { name: entry.name, points, stop, fits }
}

/// The plan, and the runner over real entries on the category's primary
/// input, or on `input` when one is named, the entries whose category lacks
/// it left out. No climb passes `max_size`.
/// A run at a size is a batch sized by the entry's previous time, seeded by
/// the size, under the no op logger, and its cost is the wall time of the
/// whole measurement.
#[derive(Clone, Copy, Debug)]
pub struct Sweep {
    pub budget: Duration,
    pub share: u32,
    pub input: Option<&'static str>,
    pub max_size: usize,
}

impl Sweep {
    /// The input `entry` runs on, `None` when its category has no such shape.
    pub fn input_for(&self, entry: &AlgorithmEntry) -> Option<&'static str> {
        match self.input {
            None => Some(primary_input(entry.category)),
            Some(name) => list_inputs(entry.category).into_iter().find(|&i| i == name),
        }
    }

    pub fn run(&self, entries: &[&'static AlgorithmEntry]) -> SweepReport {
        let runnable: Vec<&'static AlgorithmEntry> =
            entries.iter().copied().filter(|e| self.input_for(e).is_some()).collect();
        let mut last: HashMap<&str, Duration> = HashMap::new();
        let mut clock = |entry: &AlgorithmEntry, n: usize| {
            let started = Instant::now();
            let m = last.get(entry.name).map_or(1, |&one| batch_size(one, n));
            let seeds: Vec<u64> = (0..m as u64).map(|i| n as u64 * 64 + i).collect();
            let run = time_runs(entry, self.input_for(entry)?, n, &seeds)?;
            last.insert(entry.name, run);
            Some(Measure { run, cost: started.elapsed() })
        };
        sweep(&runnable, self, &mut clock)
    }
}

impl SweepReport {
    /// One table, largest fitting size first: the sizes an entry climbed, the
    /// last one that fit the share with its time, and why the climb ended.
    pub fn table(&self) -> String {
        let mut rows: Vec<&Scaling> = self.scalings.iter().collect();
        rows.sort_by_key(|s| std::cmp::Reverse(s.fits));
        let width = rows.iter().map(|s| s.name.len()).fold("algorithm".len(), usize::max);
        let header = [
            format!(
                "=== sweep: {} entries, input {}, budget {:.1?}, spent {:.1?}, one run at most {:.2?} (budget / entries / {}), sizes to {} ===",
                rows.len(), self.input, self.budget, self.spent, self.limit, self.share, self.max_size
            ),
            format!("{:<width$}  {:>6}  {:>10}  {:>12}  {}", "algorithm", "sizes", "fits", "time there", "stopped by"),
            format!("{:-<width$}  {:->6}  {:->10}  {:->12}  {:->10}", "", "", "", "", ""),
        ];
        let lines = rows.iter().map(|s| {
            let at_fit = s.points.iter().rev().find(|&&(n, _)| n <= s.fits).map_or(Duration::ZERO, |&(_, d)| d);
            let stopped = match s.stop {
                Stop::Over(n) => format!("over at {n}"),
                Stop::Cap(n) => format!("cap {n}"),
                Stop::Ceiling(n) => format!("max size {n}"),
                Stop::Failed(n) => format!("failed at {n}"),
            };
            format!("{:<width$}  {:>6}  {:>10}  {:>12.2?}  {}", s.name, s.points.len(), s.fits, at_fit, stopped)
        });
        header.into_iter().chain(lines).collect::<Vec<_>>().join("\n")
    }

    /// The archive: a header naming the budget, the time spent, the share and the ceiling, one
    /// `name<TAB>size<TAB>nanoseconds` line per timed size sorted by name then
    /// size, and a `# fits` line per entry with its largest fitting size.
    pub fn archive(&self) -> String {
        let header = format!(
            "# sweep budget_ms={} spent_ms={} share={} limit_ns={} input={} max_size={}\n",
            self.budget.as_millis(), self.spent.as_millis(), self.share, self.limit.as_nanos(), self.input, self.max_size
        );
        let by_name: BTreeMap<&str, &Scaling> = self.scalings.iter().map(|s| (s.name, s)).collect();
        let points = by_name.values().flat_map(|s| s.points.iter().map(move |&(n, d)| format!("{}\t{n}\t{}\n", s.name, d.as_nanos())));
        let fits = by_name.values().map(|s| format!("# fits\t{}\t{}\n", s.name, s.fits));
        header + &points.chain(fits).collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench_registry::{Category, RunConfig};
    use array_vis_bench_traits::Complexity;
    use sort_logger::SortLogger;

    fn no_run(_: &str, _: &RunConfig, _: &mut dyn SortLogger<usize>) {}
    fn no_sort(_: &mut [usize], _: usize, _: &mut dyn SortLogger<usize>) {}
    fn no_battery() {}

    const fn entry(name: &'static str, cap: Option<usize>) -> AlgorithmEntry {
        AlgorithmEntry {
            name,
            category: Category::Sort,
            worst: Complexity::UNKNOWN,
            best: Complexity::UNKNOWN,
            average: Complexity::UNKNOWN,
            space: Complexity::UNKNOWN,
            stable: false,
            adaptive: false,
            nondeterministic: false,
            max_input_size: cap,
            menu: &[],
            axes: &[],
            label_template: "",
            run: no_sort,
            run_with_input: no_run,
            run_correctness: no_battery,
        }
    }

    static LINEAR: AlgorithmEntry = entry("linear", None);
    static CAPPED: AlgorithmEntry = entry("capped", Some(150));
    static FLAKY: AlgorithmEntry = entry("flaky", None);

    // A plan over the primary inputs with no ceiling.
    fn plan(budget: Duration, share: u32) -> Sweep {
        Sweep { budget, share, input: None, max_size: usize::MAX }
    }

    // A clock where a run at size n takes n microseconds and costs only that.
    fn micros_per_element(_: &AlgorithmEntry, n: usize) -> Option<Measure> {
        let run = Duration::from_micros(n as u64);
        Some(Measure { run, cost: run })
    }

    #[test]
    fn the_ladder_is_dense_then_tens_then_hundreds_then_powers_of_two() {
        let first: Vec<usize> = sizes().take(122).collect();
        assert_eq!(&first[..100], &(1..=100).collect::<Vec<_>>()[..]);
        assert_eq!(&first[100..110], &[110, 120, 130, 140, 150, 160, 170, 180, 190, 200]);
        assert_eq!(&first[110..118], &[300, 400, 500, 600, 700, 800, 900, 1000]);
        assert_eq!(&first[118..], &[2048, 4096, 8192, 16384]);
    }

    #[test]
    fn a_climb_stops_at_the_first_run_over_the_share_and_the_search_finds_the_edge() {
        // One entry, share 5: the limit is 500us, so sizes to 500 fit and 600 is over.
        let report = sweep(&[&LINEAR], &plan(Duration::from_micros(2500), 5), &mut micros_per_element);
        assert_eq!(report.limit, Duration::from_micros(500));
        let [s] = &report.scalings[..] else { panic!("one scaling") };
        assert_eq!(s.stop, Stop::Over(600));
        assert_eq!(s.points.last(), Some(&(600, Duration::from_micros(600))));
        assert_eq!(s.points.len(), 100 + 10 + 4);
        // The climb spent well over the budget, so no search ran and the last
        // ladder size that fit stands.
        assert_eq!(s.fits, 500);
    }

    #[test]
    fn the_search_converges_on_the_largest_size_that_fits() {
        // Share 40 keeps the limit at 500us while the climb to 600, about 8.4ms
        // of the 20ms budget, leaves the search between 500 and 600 the seven
        // runs it needs. On this clock the edge is the limit itself.
        let mut searched = Vec::new();
        let mut clock = |e: &AlgorithmEntry, n: usize| {
            if (500..600).contains(&n) && n != 500 { searched.push(n); }
            micros_per_element(e, n)
        };
        let report = sweep(&[&LINEAR], &plan(Duration::from_millis(20), 40), &mut clock);
        assert_eq!(report.limit, Duration::from_micros(500));
        let s = &report.scalings[0];
        assert_eq!(s.stop, Stop::Over(600));
        assert_eq!(s.fits, 500);
        assert_eq!(searched, [550, 525, 512, 506, 503, 501]);
        // The search's runs are points too, so the climb reads in size order.
        let sizes_after_500: Vec<usize> = s.points.iter().map(|p| p.0).skip_while(|&n| n < 500).collect();
        assert_eq!(sizes_after_500, [500, 501, 503, 506, 512, 525, 550, 600]);
    }

    #[test]
    fn the_budget_pays_for_building_the_inputs_too() {
        // The search test's setup, but a measurement costs ten times its run,
        // as when building the input dwarfs a fast run. The climb to 600 then
        // costs about 84ms of the 20ms budget, so no search runs, and the
        // report says what was spent.
        let mut searched = Vec::new();
        let mut clock = |e: &AlgorithmEntry, n: usize| {
            if (501..600).contains(&n) { searched.push(n); }
            micros_per_element(e, n).map(|m| Measure { run: m.run, cost: m.run * 10 })
        };
        let report = sweep(&[&LINEAR], &plan(Duration::from_millis(20), 40), &mut clock);
        let s = &report.scalings[0];
        assert_eq!((s.stop, s.fits), (Stop::Over(600), 500));
        assert!(searched.is_empty(), "{searched:?}");
        let climbed: Duration = s.points.iter().map(|&(_, d)| d * 10).sum();
        assert_eq!(report.spent, climbed);
    }

    #[test]
    fn a_cap_ends_the_climb_without_a_search() {
        let report = sweep(&[&CAPPED], &plan(Duration::from_secs(1), 5), &mut micros_per_element);
        let s = &report.scalings[0];
        assert_eq!(s.stop, Stop::Cap(150));
        assert_eq!(s.points.last().map(|p| p.0), Some(150));
        assert_eq!(s.fits, 150);
    }

    #[test]
    fn a_run_that_does_not_grow_climbs_only_to_the_ceiling() {
        // A no op entry: every run takes a microsecond, so no run ever
        // exceeds the limit and only the ceiling ends the climb, at the last
        // ladder size under it. A cap at or below the ceiling still reads as
        // the cap.
        let mut clock = |_: &AlgorithmEntry, _: usize| {
            let run = Duration::from_micros(1);
            Some(Measure { run, cost: run })
        };
        let ceiled = Sweep { max_size: 5000, ..plan(Duration::from_secs(1), 5) };
        let s = &sweep(&[&LINEAR], &ceiled, &mut clock).scalings[0];
        assert_eq!((s.stop, s.fits), (Stop::Ceiling(5000), 4096));
        assert_eq!(s.points.last().map(|p| p.0), Some(4096));
        let at_cap = Sweep { max_size: 150, ..ceiled };
        assert_eq!(sweep(&[&CAPPED], &at_cap, &mut clock).scalings[0].stop, Stop::Cap(150));
    }

    #[test]
    fn a_failed_run_ends_the_climb_at_the_last_good_size() {
        let mut clock = |e: &AlgorithmEntry, n: usize| if n == 50 { None } else { micros_per_element(e, n) };
        let report = sweep(&[&FLAKY], &plan(Duration::from_secs(1), 5), &mut clock);
        let s = &report.scalings[0];
        assert_eq!(s.stop, Stop::Failed(50));
        assert_eq!((s.points.len(), s.fits), (49, 49));
    }

    #[test]
    fn the_limit_is_the_budget_over_entries_times_share() {
        // Two entries at share 5: ten runs' worth, so the limit is 840us, 800
        // fits and 900 is over. The two climbs cost far more than the budget,
        // so no search runs and both fits stay at the ladder.
        let mut searched = 0;
        let mut clock = |e: &AlgorithmEntry, n: usize| {
            if (801..900).contains(&n) { searched += 1; }
            micros_per_element(e, n)
        };
        let report = sweep(&[&LINEAR, &FLAKY], &plan(Duration::from_micros(8400), 5), &mut clock);
        assert_eq!(report.limit, Duration::from_micros(840));
        for s in &report.scalings {
            assert_eq!((s.stop, s.fits), (Stop::Over(900), 800), "{}", s.name);
        }
        assert_eq!(searched, 0);
    }

    #[test]
    fn the_archive_lists_points_by_name_then_size_under_a_header() {
        let report = sweep(&[&CAPPED], &plan(Duration::from_millis(1), 2), &mut micros_per_element);
        let text = report.archive();
        assert!(text.starts_with(&format!("# sweep budget_ms=1 spent_ms=5 share=2 limit_ns=500000 input=primary max_size={}\n", usize::MAX)));
        assert!(text.contains("capped\t1\t1000\ncapped\t2\t2000\n"));
        assert!(text.ends_with("# fits\tcapped\t150\n"));
    }

    #[test]
    fn the_table_ranks_by_the_largest_fitting_size() {
        let report = sweep(&[&CAPPED, &LINEAR], &plan(Duration::from_millis(10), 1), &mut micros_per_element);
        let table = report.table();
        let linear = table.find("linear").unwrap();
        let capped = table.find("capped").unwrap();
        assert!(linear < capped, "{table}");
        assert!(table.contains("cap 150"));
    }
}
