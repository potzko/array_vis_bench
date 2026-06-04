//! Build-time codegen step. Reads the text registry, enumerates legal sorts,
//! and writes a `generated::SORTS` table into OUT_DIR. This is the concrete
//! "more than one compile step" — codegen runs here, then rustc compiles the
//! result. Same engine as the macro and the standalone generator.

use std::{env, fs, path::PathBuf};

fn main() {
    // Registry lives at the workspace root, one level up from this crate.
    let registry_path = "../registry.spec";
    println!("cargo:rerun-if-changed={registry_path}");
    println!("cargo:rerun-if-changed=build.rs");

    let text = fs::read_to_string(registry_path).expect("read registry");
    let reg = spec_core::Registry::parse(&text).expect("parse registry");
    let specs = spec_core::enumerate(&reg, "Sort", 5);
    let code = spec_core::generate_table(&reg, &specs, "generated").expect("generate table");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("generated_sorts.rs");
    fs::write(out, code).expect("write generated_sorts.rs");
}
