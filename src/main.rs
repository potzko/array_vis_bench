//! The default `array_vis_bench` visualiser binary: a link root over the spec
//! catalog.
//!
//! It anchors `spec_catalog`, so the registry the shared CLI folds over holds
//! exactly the spec emitted entries, and calls `array_vis_bench_cli::run`. The
//! aux dev tools in `src/bin/` build alongside it.
//!
//! `shell_sort_lib` arrives through `spec_catalog` with its self registration
//! feature off, so the shell sorts are the catalog's own emitted entries.

// Anchor the unified spec catalog so the linker keeps it: its entries
// self-register (linkme + #[ctor]) into ALGORITHMS and the picker tree. One
// crate now emits every family across all kinds (sorts + quick-selects).
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
    // `array_vis_bench --timing [N] [K]` runs the benchmark consumer over every
    // registered entry: a random array of size N, K runs each (median), one
    // fastest first table per category. Entries whose contract cap is below N are
    // skipped. An N or K left out keeps the consumer's default.
    let argv: Vec<String> = std::env::args().collect();
    if let Some(pos) = argv.iter().position(|a| a == "--timing") {
        let arg = |offset: usize| argv.get(pos + offset).and_then(|s| s.parse().ok());
        let report = spec_catalog::run_timing(arg(1), arg(2));
        std::process::exit(if report.all_ok() { 0 } else { 1 });
    }

    array_vis_bench_cli::run();
}
