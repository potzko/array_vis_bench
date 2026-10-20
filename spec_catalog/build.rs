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
//! `spec` fragment, parses each under its owning crate and merges them into ONE
//! `spec_core` registry (which rejects a duplicate component name, the merged
//! namespace is global), then lowers each `query` INDEPENDENTLY (so a family's
//! helper bindings stay scoped to its own file), solves, and emits the entries
//! in a single `emit_entries` call.
//!
//! For tests, a `catalog_registry()` that rebuilds the merged registry and each
//! query are also dumped to `OUT_DIR` (`catalog_registry.rs`, `q_<stem>.avbs`) so
//! the catalog-driven tests don't need to reach into sibling crates.
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
        //
        // FAIL-FAST: a misspelled key (`spc`, `querry`) would otherwise make this
        // crate's whole family vanish silently. Reject any key outside the known
        // set so a typo is a build error, not a quietly-missing family.
        if let Some(table) = avb.as_object() {
            const KNOWN: [&str; 4] = ["spec", "query", "components", "families"];
            for key in table.keys() {
                assert!(
                    KNOWN.contains(&key.as_str()),
                    "spec_catalog: crate `{}` has unknown key `{key}` in \
                     [package.metadata.array_vis_bench] — expected one of {KNOWN:?} \
                     (did you misspell `spec`/`query`?)",
                    pkg.name
                );
            }
        }
        // …and a present-but-non-string `spec`/`query` is just as silent — catch it.
        let as_str_field = |key: &str| -> Option<&str> {
            avb.get(key).map(|v| {
                v.as_str().unwrap_or_else(|| {
                    panic!(
                        "spec_catalog: crate `{}`: [package.metadata.array_vis_bench] \
                         `{key}` must be a string path, got {v}",
                        pkg.name
                    )
                })
            })
        };
        let spec_rel = as_str_field("spec");
        let query_rel = as_str_field("query");
        if spec_rel.is_none() && query_rel.is_none() {
            continue;
        }
        // FAIL-FAST: a crate that declares a fragment but isn't in our dependency
        // closure used to warn-and-skip — i.e. its family silently disappeared from
        // the catalog. Make it a hard error: add the dep or drop the metadata.
        assert!(
            in_graph.contains(&pkg.id),
            "spec_catalog: crate `{}` declares a spec/query fragment but spec_catalog \
             does not depend on it — its family would be silently dropped. Add it as a \
             dependency in spec_catalog/Cargo.toml (or remove the metadata).",
            pkg.name
        );
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

    // One registry over every fragment, each parsed under its owning crate.
    let specs: Vec<(&str, &str)> = fragments
        .iter()
        .filter_map(|f| f.spec.as_ref().map(|(_, text)| (f.krate.as_str(), text.as_str())))
        .collect();
    let reg = spec_core::Registry::from_fragments(specs.iter().copied())
        .unwrap_or_else(|e| panic!("merge the catalog fragments: {e}"));
    fs::write(out_dir.join("catalog_registry.rs"), registry_helper(&specs))
        .expect("write catalog_registry.rs");

    // ── Lower each query independently, solve, accumulate variants ──
    let mut all_entries: Vec<(spec_core::SpecNode, &str)> = Vec::new();
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

        let before = all_entries.len();
        for algo in &program.algos {
            let out = spec_core::solve(&algo.query, &reg)
                .unwrap_or_else(|e| panic!("solve family `{}` ({}): {e}", algo.name, f.krate));
            for w in &out.warnings {
                println!("cargo:warning=spec_catalog: {} — {w}", algo.name);
            }
            all_entries.extend(out.sorts.into_iter().map(|node| (node, algo.category)));
        }
        let n = all_entries.len() - before;
        // FAIL-FAST: a query that resolves to nothing is almost always a broken
        // query or a fragment that didn't merge — not an intentionally-empty
        // family. Drop it on the floor and the loss is invisible; fail instead.
        assert!(
            n > 0,
            "spec_catalog: query `{}` ({}) solved to ZERO AlgorithmEntry rows — \
             a family emitting nothing indicates a broken query or an unmerged \
             fragment; failing rather than silently dropping it.",
            stem, f.krate
        );
        total += n;
        // Informational tally (per-family row count). Plain stdout, not
        // `cargo::warning=` — captured to the build script's `output` file and
        // shown only under `cargo build -vv`, so default builds stay quiet.
        // Genuine problems (unsatisfiable query, dup name) still warn/panic.
        println!("spec_catalog: {} ({}) → {n} AlgorithmEntry rows", stem, f.krate);
        count_consts.push_str(&format!(
            "#[allow(dead_code)] const COUNT_{}: usize = {n};\n",
            stem.to_uppercase()
        ));
        // Dump the query for the catalog-driven tests (e.g. the slotted golden).
        fs::write(out_dir.join(format!("q_{stem}.avbs")), text)
            .unwrap_or_else(|_| panic!("write q_{stem}.avbs"));
    }

    println!("spec_catalog: emitting {total} AlgorithmEntry rows total");
    // FAIL-FAST: an empty catalog means the gather/merge/solve pipeline produced
    // nothing — a misconfiguration that must not compile to a working-but-empty app.
    assert!(total > 0, "spec_catalog: emitted ZERO AlgorithmEntry rows total — the \
        catalog is empty; the fragment gather/merge/solve pipeline is misconfigured.");

    let code = spec_core::emit_entries(&reg, &all_entries, &spec_emit::ArrayBackend::default())
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
                "spec_catalog: consumer `{}` → {} main(s) over {} algorithms",
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

/// The test helper that rebuilds the merged registry from the same fragments,
/// each under its owning crate.
fn registry_helper(specs: &[(&str, &str)]) -> String {
    let fragments: String =
        specs.iter().map(|(krate, text)| format!("        ({krate:?}, {text:?}),\n")).collect();
    format!(
        "#[cfg(test)]\n\
         pub(crate) fn catalog_registry() -> spec_core::Registry {{\n    \
         spec_core::Registry::from_fragments([\n{fragments}    ])\n    \
         .expect(\"merge the catalog fragments\")\n}}\n"
    )
}
