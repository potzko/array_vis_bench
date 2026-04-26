use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let result = combo_codegen::scan("src/").expect("combo_codegen scan failed");
    result.emit_rerun();
    println!("cargo:rerun-if-changed=build.rs");

    result.emit_sort_families(&out_dir).expect("emit_sort_families failed");
}
