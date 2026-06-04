//! Standalone front-end (mode 2: "run a program to generate the sorts").
//! Loads the text registry, enumerates every legal sort up to a depth bound,
//! and prints the generated Rust dispatch table to stdout.
//!
//! Run from the workspace root:  `cargo run -p generator`
//!
//! This shares the exact engine the macro uses — the only difference is the
//! front-end produces MANY trees (bounded enumeration) instead of taking one.

const REGISTRY: &str = include_str!("../../registry.spec");

fn main() {
    let max_depth: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    let reg = spec_core::Registry::parse(REGISTRY).expect("registry parses");
    let specs = spec_core::enumerate(&reg, "Sort", max_depth);
    let code = spec_core::generate_table(&reg, &specs, "generated").expect("all combos resolve");

    eprintln!("// {} sorts enumerated (max_depth = {max_depth})", specs.len());
    print!("{code}");
}
