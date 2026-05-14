use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let config = combo_codegen::CodegenConfig::for_sort_families();
    let result = combo_codegen::scan("src/", &config).expect("combo_codegen scan failed");
    result.emit_rerun();
    println!("cargo:rerun-if-changed=build.rs");

    result.emit_families(&out_dir).expect("emit_families failed");
}
