//! The default `array_vis_bench` visualiser — now a SPEC-ONLY link root.
//!
//! It anchors only the constraint-compiler crates, so the registry the shared
//! CLI folds over contains *only* spec-emitted entries — `cargo run` is purely
//! spec-oriented. The legacy combo_codegen registry (`array_vis_bench_full`) is
//! an optional dep behind the `combo-tools` feature (off by default), so it is
//! not even compiled here; the aux tools in `src/bin/` build with that feature.
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
        match spec_catalog::run_declared_consumers() {
            Ok(reports) => {
                let mut failed = false;
                for r in &reports {
                    println!("consumer `{}`: {}/{} ok", r.consumer, r.ok, r.ran);
                    for f in &r.failures {
                        failed = true;
                        println!("  FAILED: {f}");
                    }
                }
                std::process::exit(if failed { 1 } else { 0 });
            }
            Err(e) => {
                eprintln!("consumer error: {e}");
                std::process::exit(2);
            }
        }
    }
    array_vis_bench_cli::run();
}
