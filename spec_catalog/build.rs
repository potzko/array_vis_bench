//! Codegen for the UNIFIED spec catalog — fragments OWNED by building-stone
//! crates, GATHERED here via the dependency graph (the "B" architecture).
//!
//! Each building-stone crate declares, in its `Cargo.toml`:
//!
//! ```toml
//! [package.metadata.array_vis_bench]
//! spec  = "shell.spec"   # the catalog fragment this crate contributes
//! query = "shell.avbs"   # (optional) the AVBS program that builds its families
//! ```
//!
//! This build script runs `cargo metadata`, walks `spec_catalog`'s dependency
//! CLOSURE (so unrelated workspace members are ignored), reads every gathered
//! `spec` fragment, merges them into ONE `spec_core` registry (with a
//! duplicate-component-name guard — the merged namespace is global), then lowers
//! each `query` INDEPENDENTLY (so a family's helper bindings stay scoped to its
//! own file), solves, and emits the entries in a single `emit_entries` call.
//!
//! For tests, the merged registry text and each query are also dumped to
//! `OUT_DIR` (`merged_catalog.spec`, `q_<stem>.avbs`) so the catalog-driven tests
//! don't need to reach into sibling crates.
//!
//! Adding a family = give its crate a `spec`/`query` metadata block + a dep on it
//! here. If a crate declares a fragment but `spec_catalog` doesn't depend on it,
//! the script WARNS (it would otherwise silently drop the family).

use std::collections::{HashMap, HashSet};
use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(manifest_dir.join("Cargo.toml"))
        .exec()
        .expect("cargo metadata");

    // ── spec_catalog's dependency closure (so we ignore unrelated members) ──
    let resolve = metadata.resolve.as_ref().expect("cargo metadata resolve graph");
    let root_id = metadata
        .packages
        .iter()
        .find(|p| p.name == "spec_catalog")
        .map(|p| p.id.clone())
        .expect("spec_catalog present in metadata");
    let node_by_id: HashMap<_, _> = resolve.nodes.iter().map(|n| (n.id.clone(), n)).collect();
    let mut in_graph: HashSet<_> = HashSet::new();
    let mut stack = vec![root_id];
    while let Some(id) = stack.pop() {
        if !in_graph.insert(id.clone()) {
            continue;
        }
        if let Some(node) = node_by_id.get(&id) {
            stack.extend(node.dependencies.iter().cloned());
        }
    }

    // ── Gather (spec fragment, query) from each crate that declares them ──
    // Sorted by crate name for deterministic merge / module-index order.
    let mut packages: Vec<_> = metadata.packages.iter().collect();
    packages.sort_by(|a, b| a.name.cmp(&b.name));

    struct Fragment {
        krate: String,
        spec: Option<(PathBuf, String)>,
        query: Option<(PathBuf, String)>,
    }
    let mut fragments: Vec<Fragment> = Vec::new();

    for pkg in &packages {
        let avb = match pkg.metadata.get("array_vis_bench") {
            Some(v) => v,
            None => continue,
        };
        // Only the `spec`/`query` keys concern us; crates that declare ONLY the
        // legacy combo `components`/`families` arrays carry no `spec` key here.
        let spec_rel = avb.get("spec").and_then(|v| v.as_str());
        let query_rel = avb.get("query").and_then(|v| v.as_str());
        if spec_rel.is_none() && query_rel.is_none() {
            continue;
        }
        if !in_graph.contains(&pkg.id) {
            println!(
                "cargo:warning=spec_catalog: crate `{}` declares a spec fragment but \
                 spec_catalog does not depend on it — add it as a dependency to include \
                 its family",
                pkg.name
            );
            continue;
        }
        let dir = pkg.manifest_path.parent().expect("manifest has a parent dir");
        let read = |rel: &str| -> (PathBuf, String) {
            let p: PathBuf = dir.join(rel).into();
            let text = fs::read_to_string(&p)
                .unwrap_or_else(|e| panic!("read {} ({}): {e}", p.display(), pkg.name));
            println!("cargo:rerun-if-changed={}", p.display());
            (p, text)
        };
        fragments.push(Fragment {
            krate: pkg.name.to_string(),
            spec: spec_rel.map(read),
            query: query_rel.map(read),
        });
        println!("cargo:rerun-if-changed={}", pkg.manifest_path);
    }

    // ── Merge fragments → one registry text, with a duplicate-name guard ──
    let mut merged = String::new();
    let mut comp_owner: HashMap<String, String> = HashMap::new();
    for f in &fragments {
        let (path, text) = match &f.spec {
            Some(s) => s,
            None => continue,
        };
        for line in text.lines() {
            let line = line.trim_start();
            if let Some(name) = line.strip_prefix("component ") {
                let name = name.trim().to_string();
                if let Some(prev) = comp_owner.insert(name.clone(), f.krate.clone()) {
                    panic!(
                        "duplicate component `{name}` declared by both `{prev}` and `{}` \
                         (the merged catalog namespace is global)",
                        f.krate
                    );
                }
            }
        }
        merged.push_str(&format!("# ═══ from crate `{}` ({}) ═══\n", f.krate, path.display()));
        merged.push_str(text);
        merged.push_str("\n\n");
    }
    fs::write(out_dir.join("merged_catalog.spec"), &merged).expect("write merged_catalog.spec");

    let reg = spec_core::Registry::parse(&merged).expect("parse merged catalog");

    // ── Lower each query independently, solve, accumulate variants ──
    let mut all_sorts = Vec::new();
    let mut count_consts = String::new();
    let mut total = 0usize;
    for f in &fragments {
        let (path, text) = match &f.query {
            Some(q) => q,
            None => continue,
        };
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let program = spec_core::avbs::lower(text, &reg)
            .unwrap_or_else(|e| panic!("lower {}.avbs ({}): {e}", stem, f.krate));

        let before = all_sorts.len();
        for algo in &program.algos {
            let out = spec_core::solve(&algo.query, &reg)
                .unwrap_or_else(|e| panic!("solve family `{}` ({}): {e}", algo.name, f.krate));
            for w in &out.warnings {
                println!("cargo:warning=spec_catalog: {} — {w}", algo.name);
            }
            all_sorts.extend(out.sorts);
        }
        let n = all_sorts.len() - before;
        total += n;
        println!("cargo:warning=spec_catalog: {} ({}) → {n} AlgorithmEntry rows", stem, f.krate);
        count_consts.push_str(&format!(
            "#[allow(dead_code)] const COUNT_{}: usize = {n};\n",
            stem.to_uppercase()
        ));
        // Dump the query for the catalog-driven tests (e.g. the slotted golden).
        fs::write(out_dir.join(format!("q_{stem}.avbs")), text)
            .unwrap_or_else(|_| panic!("write q_{stem}.avbs"));
    }

    println!("cargo:warning=spec_catalog: emitting {total} AlgorithmEntry rows total");

    let code = spec_core::emit_entries(&reg, &all_sorts, &spec_emit::ArrayBackend::default())
        .expect("emit_entries");
    fs::write(out_dir.join("catalog_entries.rs"), code).expect("write catalog_entries.rs");

    count_consts.push_str(&format!("#[allow(dead_code)] const COUNT_TOTAL: usize = {total};\n"));
    fs::write(out_dir.join("catalog_counts.rs"), count_consts).expect("write catalog_counts.rs");

    // ── The catalog's AVBS CONSUMER program (`consumers.avbs`) ──
    // Lower it against the SAME merged registry, resolve each `consumers:
    // List<Mains<Set>>` to (main names, concrete algorithm labels), and emit a
    // `run_declared_consumers()` that runs them via `mains::run_named` — the
    // language's consumer surface driving the REAL emitted catalog. The set may
    // mix kinds (sorts + quick-selects); `correctness` is kind-agnostic.
    let consumers_path = manifest_dir.join("consumers.avbs");
    println!("cargo:rerun-if-changed={}", consumers_path.display());
    let consumer_code = if consumers_path.exists() {
        let text = fs::read_to_string(&consumers_path).expect("read consumers.avbs");
        let program = spec_core::avbs::lower(&text, &reg).expect("lower consumers.avbs");
        let resolved = program.resolve_consumers(&reg).expect("resolve consumers.avbs");
        let mut runs = String::new();
        let mut shape = String::new();
        for c in &resolved {
            let mains_lit: String = c.mains.iter().map(|m| format!("{m:?}, ")).collect();
            let labels_lit: String = c.algo_labels.iter().map(|l| format!("{l:?}, ")).collect();
            runs.push_str(&format!(
                "    out.extend(array_vis_bench_core::mains::run_named(&[{mains_lit}], &[{labels_lit}])?);\n"
            ));
            shape.push_str(&format!(
                "({:?}, {}, {}), ",
                c.name,
                c.mains.len(),
                c.algo_labels.len()
            ));
            println!(
                "cargo:warning=spec_catalog: consumer `{}` → {} main(s) over {} algorithms",
                c.name,
                c.mains.len(),
                c.algo_labels.len()
            );
        }
        format!(
"/// AVBS-defined consumer pipelines from `consumers.avbs`, resolved at build time
/// to concrete (main names, algorithm labels). Running them drives the language's
/// `consumers: List<Mains<Set>>` surface over the REAL registered catalog.
pub fn run_declared_consumers(
) -> Result<Vec<array_vis_bench_core::mains::MainReport>, String> {{
    let mut out = Vec::new();
{runs}    Ok(out)
}}

/// `(consumer name, #mains, #algorithm labels)` per declared consumer — for tests.
pub const DECLARED_CONSUMER_SHAPE: &[(&str, usize, usize)] = &[{shape}];
"
        )
    } else {
        String::new()
    };
    fs::write(out_dir.join("declared_consumers.rs"), consumer_code)
        .expect("write declared_consumers.rs");
}
