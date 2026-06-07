//! Build-time codegen step (the extra "compile step"). Reads the text registry
//! and runs the typed-constraint SOLVER on a handful of queries, concatenating
//! their ground sorts into one `generated::SORTS` dispatch table in OUT_DIR.
//!
//! THE PAYOFF vs. the old prototype: the previous build.rs had to DROP the whole
//! `quick_sort` family before emitting, because flat enumeration overproduced
//! arity-mismatched combos (single partition + dual pivot) the registry could
//! not pre-filter. Here the quick_sort family is generated through a SHARED
//! PIVOT VARIABLE — the LL-partition + dual-pivot combination is never built —
//! so it is emitted in full, arity-correct, nothing dropped. rustc still
//! type-checks every row as the redundant final gate.

use std::{env, fs, path::PathBuf};

/// Each query is a family; together they tile the catalog. The same evaluator
/// (`spec_core::solve`) handles the fully-pinned, partial, and full cases.
const QUERIES: &[(&str, &str)] = &[
    // quick_sort: arity made structural via the shared `p`. 7 pivots, each
    // pinned to its matching partition, × 3 small sorts = 21 — all legal.
    (
        "quick",
        "let p: Pivot = .;
         let part: Partition[pivot = p] = .;
         let s: Sort = quick_sort(partition = part, pivot = p, small_sort = .);",
    ),
    // merge: a partial family over the small-sort axis, bool consts pinned.
    (
        "merge",
        "let m: Sort = top_down_merge(small_sort = ., ping_pong = true, early_exit = false);",
    ),
    // shell: exhaust the gap sequences.
    ("shell", "let sh: Sort = shell_sort(seq = .);"),
    // heap: enumerate the declared d-ary arity set {2,3,4} — const value holes.
    ("heap", "let h: Sort = heap_sort(arity = *);"),
    // recursion: a sort whose inner is a sort, bounded by the depth knob.
    ("recursive", "depth 3; let r: RecSort = .;"),
];

fn main() {
    let registry_path = "../registry.spec";
    println!("cargo:rerun-if-changed={registry_path}");
    println!("cargo:rerun-if-changed=build.rs");

    let text = fs::read_to_string(registry_path).expect("read registry");
    let reg = spec_core::Registry::parse(&text).expect("parse registry");

    let mut all = Vec::new();
    for (name, query) in QUERIES {
        let q = spec_core::parse_query(query).unwrap_or_else(|e| panic!("query `{name}`: {e}"));
        let out = spec_core::solve(&q, &reg).unwrap_or_else(|e| panic!("solve `{name}`: {e}"));
        for w in &out.warnings {
            println!("cargo:warning={name}: {w}");
        }
        println!("cargo:warning={name}: {} sorts", out.sorts.len());
        all.extend(out.sorts);
    }

    let quick = all.iter().filter(|s| s.name == "quick_sort").count();
    println!(
        "cargo:warning=emitted {} sorts incl. the FULL quick_sort family ({quick} variants, \
         arity-correct, none dropped)",
        all.len()
    );

    let code = spec_core::generate_table(&reg, &all, "generated").expect("generate table");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("generated_sorts.rs"), code).expect("write generated_sorts.rs");

    // ── Phase 0: the REAL emit target — AlgorithmEntry rows for the SORT
    //    families, registered into avb_abi::ALGORITHMS. Same evaluator, new
    //    backend (`emit_entries` instead of the toy `generate_table`).
    let mut entries = Vec::new();
    for (name, query) in ENTRY_QUERIES {
        let q = spec_core::parse_query(query).unwrap_or_else(|e| panic!("entry query `{name}`: {e}"));
        let out = spec_core::solve(&q, &reg).unwrap_or_else(|e| panic!("solve `{name}`: {e}"));
        entries.extend(out.sorts);
    }
    let entry_code = spec_core::emit_entries(&reg, &entries, &spec_core::EmitConfig::default())
        .expect("emit_entries");
    println!("cargo:warning=emitted {} AlgorithmEntry rows", entries.len());
    fs::write(out_dir.join("generated_entries.rs"), entry_code).expect("write generated_entries.rs");
}

/// Families emitted as real `AlgorithmEntry` rows. The first four are the SORT
/// category; the last three exercise the Partition/Merge/Rotation emit drivers
/// — same evaluator, different per-category `run_with_input`/battery bodies. A
/// bare role hole (`let pa: PartitionAlgo = .;`) enumerates every component
/// providing that role.
const ENTRY_QUERIES: &[(&str, &str)] = &[
    (
        "quick",
        "let p: Pivot = .;
         let part: Partition[pivot = p] = .;
         let s: Sort = quick_sort(partition = part, pivot = p, small_sort = .);",
    ),
    (
        "merge",
        "let m: Sort = top_down_merge(small_sort = ., ping_pong = true, early_exit = false);",
    ),
    ("shell", "let sh: Sort = shell_sort(seq = .);"),
    ("heap", "let h: Sort = heap_sort(arity = *);"),
    ("partition", "let pa: PartitionAlgo = .;"),
    ("merge_op", "let mo: MergeAlgo = .;"),
    ("rotation", "let ro: RotationAlgo = .;"),
];
