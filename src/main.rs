//! The default `array_vis_bench` visualiser — now a SPEC-ONLY link root.
//!
//! It anchors only the constraint-compiler crates, so the registry the shared
//! CLI folds over contains *only* spec-emitted entries — `cargo run` is purely
//! spec-oriented. The legacy `combo_codegen` cross-product generator and its
//! wiring crate have been retired; the spec catalog is now the sole source of
//! `ALGORITHMS`. The aux dev tools in `src/bin/` build by default.
//!
//! Purely spec: `shell_sort_lib` is pulled in (via `spec_catalog`) with its
//! self-registration disabled (`default-features = false`), so there is no combo
//! "bleed" — the shell sorts are the spec compiler's own emitted entries.

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
    array_vis_bench_cli::run();
}
