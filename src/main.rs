//! The default `array_vis_bench` visualiser binary: a link root over the spec
//! catalog.
//!
//! It anchors `spec_catalog`, so the registry the shared CLI folds over holds
//! exactly the spec emitted entries, and calls `array_vis_bench_cli::run`. The
//! aux dev tools in `src/bin/` build alongside it.

// Anchor the unified spec catalog so the linker keeps it: its entries join
// ALGORITHMS through linkme. One crate emits every family across all kinds
// (sorts + quick-selects).
use spec_catalog as _;

fn main() {
    // `array_vis_bench --consumers` runs the catalog's AVBS-defined consumer
    // program (`spec_catalog/consumers.avbs`, a `consumers: List<Mains<Set>>`)
    // over the registered catalog and prints one report per consumer — the
    // language's consumer surface as the app's entry point. Default = the
    // interactive picker.
    if std::env::args().any(|a| a == "--consumers") {
        // Run the correctness (tests) and benchmark consumers over the SAME set:
        // the entire emitted catalog. The set is the emitted catalog itself, not
        // a query — see `spec_catalog::run_over_catalog`.
        match spec_catalog::run_over_catalog(&["correctness", "benchmark"]) {
            Ok(reports) => {
                println!("=== consumers over the full catalog ===");
                let mut issues = 0usize;
                let mut set = 0usize;
                for r in &reports {
                    let n = r.failures.len();
                    issues += n;
                    set = set.max(r.ran); // each consumer runs over the same full set
                    println!(
                        "consumer `{}`: {}/{} ok ({} issue{})",
                        r.consumer, r.ok, r.ran, n, if n == 1 { "" } else { "s" }
                    );
                    for f in &r.failures {
                        println!("    FAILED: {f}");
                    }
                }

                // Final report: total issues across every consumer over the set.
                println!("\n--- total ---");
                println!(
                    "{} consumer{} over {set} algorithms — {issues} total issue{}",
                    reports.len(),
                    if reports.len() == 1 { "" } else { "s" },
                    if issues == 1 { "" } else { "s" }
                );
                std::process::exit(if issues > 0 { 1 } else { 0 });
            }
            Err(e) => {
                eprintln!("consumer error: {e}");
                std::process::exit(2);
            }
        }
    }
    // `array_vis_bench --timing [N] [K] [--out PATH]` runs the benchmark
    // consumer over every registered entry: a random array of size N, K runs
    // each (median), one fastest first table per category. Entries whose
    // contract cap is below N are skipped. An N or K left out keeps the
    // consumer's default. `--out` also writes the medians as a TSV archive,
    // the input `verify.sh` compares against its timing baseline.
    // `array_vis_bench --sweep SECONDS [--share K] [--only PREFIX] [--input NAME] [--out PATH]`
    // runs the scaling sweep: every entry (or those whose name starts with
    // PREFIX) climbs the size ladder until one run takes more than
    // SECONDS / (entries * K), K defaulting to 5, then the time left finds
    // each entry's largest size that fits. `--out` writes the archive
    // `plot_bench.py` charts.
    let argv: Vec<String> = std::env::args().collect();
    let flag = |name: &str| argv.iter().position(|a| a == name).and_then(|i| argv.get(i + 1));
    // `--input NAME` picks the input shape by its registered name, for the
    // categories that have one; the others are skipped and counted.
    let input: Option<&'static str> = flag("--input").map(|name| &*Box::leak(name.clone().into_boxed_str()));
    if let Some(seconds) = flag("--sweep") {
        let seconds: f64 = seconds.parse().unwrap_or_else(|_| panic!("--sweep takes seconds, got {seconds}"));
        let share = flag("--share").map_or(5, |k| k.parse().unwrap_or_else(|_| panic!("--share takes a count, got {k}")));
        let report = spec_catalog::run_sweep(std::time::Duration::from_secs_f64(seconds), share, flag("--only").map(String::as_str), input);
        println!("{}", report.table());
        if let Some(out) = flag("--out") {
            std::fs::write(out, report.archive()).unwrap_or_else(|e| panic!("write sweep archive {out}: {e}"));
        }
        std::process::exit(0);
    }
    if let Some(pos) = argv.iter().position(|a| a == "--timing") {
        let arg = |offset: usize| argv.get(pos + offset).and_then(|s| s.parse().ok());
        let report = spec_catalog::run_timing(arg(1), arg(2), input);
        if let Some(out) = argv.iter().position(|a| a == "--out").and_then(|i| argv.get(i + 1)) {
            std::fs::write(out, report.timings_tsv())
                .unwrap_or_else(|e| panic!("write timing archive {out}: {e}"));
        }
        std::process::exit(if report.all_ok() { 0 } else { 1 });
    }

    array_vis_bench_cli::run();
}
