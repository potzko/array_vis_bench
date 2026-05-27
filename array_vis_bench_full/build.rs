use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    let config = combo_codegen::CodegenConfig::for_sort_families();
    let mut result = combo_codegen::scan("src/", &config).expect("combo_codegen scan failed");

    // Walk the dep graph (current crate + every transitive dep) for
    // components declared via `[[package.metadata.array_vis_bench.components]]`.
    // This is the discovery primitive that lets per-leaf crates work —
    // each leaf carries its own metadata block, and the wiring crate
    // picks them all up here without naming them explicitly.
    //
    // Iterating in reverse + `add_front` keeps TOML declaration order
    // intact at the front of each role's vec.
    let manifest_path = manifest_dir.join("Cargo.toml");
    let metadata_components = combo_codegen::scan_workspace_components(&manifest_path)
        .expect("combo_codegen metadata scan failed");
    for c in metadata_components.iter().rev() {
        result.registry.add_front_with_uses(
            c.role.clone(),
            c.type_expr.clone(),
            c.label.clone(),
            c.uses.clone(),
        );
    }

    // Same dep-graph walk, but for families. TOML-declared families
    // coexist with text-scanned ones; the metadata scanner finds them
    // in `[[package.metadata.array_vis_bench.families]]` blocks on any
    // crate in the wiring graph. Append to the existing families list
    // — `emit_families` groups by `source_module`, so a TOML family
    // with `module = "quick_sorts"` merges into the same generated
    // file as the in-source `family!(…)` calls already do.
    let metadata_families = combo_codegen::scan_workspace_families(&manifest_path)
        .expect("combo_codegen family metadata scan failed");
    for f in &metadata_families {
        result.families.push(f.family.clone());
    }

    // Re-run when any metadata-bearing manifest changes (dedup —
    // the same crate's manifest may contribute multiple components
    // and / or families).
    let mut seen_manifests: Vec<PathBuf> = Vec::new();
    for c in &metadata_components {
        if !seen_manifests.contains(&c.source_manifest) {
            println!("cargo:rerun-if-changed={}", c.source_manifest.display());
            seen_manifests.push(c.source_manifest.clone());
        }
    }
    for f in &metadata_families {
        if !seen_manifests.contains(&f.source_manifest) {
            println!("cargo:rerun-if-changed={}", f.source_manifest.display());
            seen_manifests.push(f.source_manifest.clone());
        }
    }

    // Catch duplicate-component and empty-axis-role mistakes before
    // emission — these would otherwise surface as missing menu entries
    // or duplicated cross-product variants well downstream.
    result.validate().expect("component / family validation failed");

    // Orphan roles aren't necessarily bugs (hand-written code can iterate
    // a role's registry directly — see `partitions_standalone.rs`) but
    // they're a common shape for typos in `role = "…"`. Surface as
    // `cargo:warning` so they show up in the build log without breaking
    // it.
    for role in result.orphan_roles() {
        println!(
            "cargo:warning=role `{role}` has components registered but no \
             family axis references it — typo in a `role = \"…\"` field, \
             or intentionally consumed outside the family system?"
        );
    }

    result.emit_rerun();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", manifest_path.display());

    result.emit_families(&out_dir).expect("emit_families failed");
}
