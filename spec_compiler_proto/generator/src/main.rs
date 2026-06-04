//! Standalone front-end (mode 2: "run a program to generate the sorts"). Loads
//! the text registry, SOLVES a constraint-language query, and prints the
//! generated Rust dispatch table to stdout.
//!
//! Run from the workspace root:
//!   cargo run -p generator                       # the default arity-safe family
//!   cargo run -p generator -- 'let s: Sort = .;' # any query as one argument
//!
//! This shares the exact engine the macro and build.rs use — the only
//! difference is the front-end: one tree (the macro), or a query that lowers to
//! a SET of trees (here). Holes are the only thing that varies.

const REGISTRY: &str = include_str!("../../registry.spec");

/// The arity-safe quick_sort family: a shared pivot variable means the
/// LL-partition + dual-pivot combination is never even built.
const DEFAULT_QUERY: &str = "let p: Pivot = .;
let part: Partition[pivot = p] = .;
let s: Sort = quick_sort(partition = part, pivot = p, small_sort = .);";

fn main() {
    let query = std::env::args().nth(1).unwrap_or_else(|| DEFAULT_QUERY.to_string());

    let reg = spec_core::Registry::parse(REGISTRY).expect("registry parses");
    let q = spec_core::parse_query(&query).expect("query parses");
    let out = spec_core::solve(&q, &reg).expect("query solves");

    for w in &out.warnings {
        eprintln!("// warning: {w}");
    }
    eprintln!("// {} sort(s) solved (depth = {})", out.sorts.len(), q.depth);

    let code = spec_core::generate_table(&reg, &out.sorts, "generated").expect("generate table");
    print!("{code}");
}
