//! Build-time codegen step. Reads the text registry, enumerates legal sorts,
//! and writes a `generated::SORTS` table into OUT_DIR.
//!
//! IMPORTANT FINDING demonstrated here: we enumerate `Sort` but DROP the
//! `quick_sort` family before emitting, because flat enumeration produces
//! arity-mismatched combos (single partition + dual pivot) that the registry
//! cannot pre-filter — they'd fail to compile. Merge + shell have no cross-slot
//! constraint, so they enumerate cleanly. See README "Findings".

use std::{env, fs, path::PathBuf};

fn main() {
    let registry_path = "../registry.spec";
    println!("cargo:rerun-if-changed={registry_path}");
    println!("cargo:rerun-if-changed=build.rs");

    let text = fs::read_to_string(registry_path).expect("read registry");
    let reg = spec_core::Registry::parse(&text).expect("parse registry");

    let mut specs = spec_core::enumerate(&reg, "Sort", 5);
    let before = specs.len();
    specs.retain(|s| s.name != "quick_sort");
    let dropped = before - specs.len();
    println!("cargo:warning=enumerated {before} Sort variants, dropped {dropped} quick_sort (flat arity cannot be pre-filtered)");

    let code = spec_core::generate_table(&reg, &specs, "generated").expect("generate table");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("generated_sorts.rs");
    fs::write(out, code).expect("write generated_sorts.rs");
}
