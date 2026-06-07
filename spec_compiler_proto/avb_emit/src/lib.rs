//! The array-domain emit backend: turns a `spec_core`-resolved algorithm type
//! into a runtime registry entry (`avb_abi::AlgorithmEntry`) registered into
//! `avb_abi::ALGORITHMS`. This is the domain half of the emit seam — `spec_core`
//! emits only the structural module scaffold (the import-scoped `mod`, its
//! `use`s, the `pub type Ty`) and defers the registration body to
//! [`ArrayBackend`] through [`spec_core::EmitBackend`].
//!
//! It depends on `spec_core` (for the seam trait + [`spec_core::EntryCtx`]) but
//! NOT on `avb_abi`: the ABI is referenced purely as text (`avb_abi::…`) in the
//! emitted source, so the dependency arrows point one way (avb_emit → spec_core)
//! and the runtime ABI crate stays free of any compile-time machinery. When
//! porting to the live repo, the `abi` path becomes the real
//! `array_vis_bench_core::bench_registry` / `…_traits` split.

use spec_core::{EmitBackend, EntryCtx};

mod drivers;
use drivers::{driver, DriverCtx};

/// The array-visualisation domain's emit backend. Carries the ABI crate path
/// the emitted code targets (`avb_abi` in the prototype).
pub struct ArrayBackend {
    pub abi: String,
}

impl Default for ArrayBackend {
    fn default() -> Self {
        ArrayBackend { abi: "avb_abi".to_string() }
    }
}

impl EmitBackend for ArrayBackend {
    /// Render the registration body spliced inside the compiler's
    /// `mod __entry_N { … pub type Ty = …; <HERE> }`. The returned source is the
    /// module's interior items, indented one level (4 spaces) and
    /// newline-terminated, so the compiler can append the closing `}` verbatim.
    ///
    /// Per-type properties (`worst/best/average/space/stable`) are *inherited*
    /// from `Ty` via `<Ty as Has*>::CONST`; `category`/`adaptive`/`max_input`
    /// come from the catalog (carried on `ctx`); the category-specific
    /// `run_with_input`/battery bodies come from the [`drivers`].
    fn entry_body(&self, ctx: &EntryCtx) -> Result<String, String> {
        let abi = &self.abi;
        // A component with no declared category defaults to Sort. That default
        // is the domain's call — the compiler treats `category` as opaque.
        let category = ctx.category.unwrap_or("Sort");
        let code = driver(category, &DriverCtx { abi })?;
        let max_input = match ctx.max_input {
            Some(n) => format!("Some({n})"),
            None => "None".to_string(),
        };
        Ok(format!(
            "\x20   fn run_default(input_name: &str, config: &{abi}::RunConfig, logger: &mut dyn {abi}::SortLogger<usize>) {{\n\
             \x20       {run_default}\n\
             \x20   }}\n\
             \x20   fn run_correct() {{\n\
             \x20       {run_correct}\n\
             \x20   }}\n\
             \x20   #[linkme::distributed_slice({abi}::ALGORITHMS)]\n\
             \x20   #[allow(non_upper_case_globals)]\n\
             \x20   static ENTRY: {abi}::AlgorithmEntry = {abi}::AlgorithmEntry {{\n\
             \x20       name: {label:?},\n\
             \x20       category: {abi}::Category::{cat},\n\
             \x20       worst: <Ty as {abi}::HasTimeBounds>::WORST,\n\
             \x20       best: <Ty as {abi}::HasTimeBounds>::BEST,\n\
             \x20       average: <Ty as {abi}::HasTimeBounds>::AVERAGE,\n\
             \x20       space: <Ty as {abi}::HasSpace>::SPACE,\n\
             \x20       stable: <Ty as {abi}::HasStability>::STABLE,\n\
             \x20       adaptive: {adaptive},\n\
             \x20       max_input_size: {max_input},\n\
             \x20       run_with_input: run_default,\n\
             \x20       run_correctness: run_correct,\n\
             \x20   }};\n\
             \x20   #[ctor::ctor]\n\
             \x20   #[allow(non_snake_case)]\n\
             \x20   fn register() {{ {abi}::register_sort_variant({label:?}, &[{menu:?}], &[]); }}\n",
            label = ctx.label,
            cat = category,
            adaptive = ctx.adaptive,
            run_default = code.run_default_body,
            run_correct = code.run_correct_body,
            menu = menu_root(category),
        ))
    }
}

/// The picker-tree root each category registers under. Only ever reached for a
/// category `drivers::driver` already accepted (`entry_body` calls `driver(…)?`
/// first), so these four are exhaustive — a fifth would fail loudly here,
/// keeping this map in lockstep with the driver dispatch.
fn menu_root(category: &str) -> &'static str {
    match category {
        "Sort" => "sorts",
        "Partition" => "partitions",
        "Merge" => "merges",
        "Rotation" => "rotations",
        other => unreachable!(
            "menu_root(`{other}`) — drivers::driver would have rejected it before this point"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_body_wraps_driver_with_inherited_consts_and_category() {
        let body = ArrayBackend::default()
            .entry_body(&EntryCtx {
                label: "shell[knuth]",
                category: Some("Sort"),
                adaptive: false,
                max_input: None,
            })
            .unwrap();
        // structural: registers into ALGORITHMS, inherits complexity from `Ty`
        assert!(body.contains("#[linkme::distributed_slice(avb_abi::ALGORITHMS)]"));
        assert!(body.contains("worst: <Ty as avb_abi::HasTimeBounds>::WORST"));
        assert!(body.contains("stable: <Ty as avb_abi::HasStability>::STABLE"));
        // catalog-supplied facets thread through verbatim
        assert!(body.contains("category: avb_abi::Category::Sort"));
        assert!(body.contains("max_input_size: None"));
        // the Sort driver body + the menu path for the category
        assert!(body.contains("run_sort_with_input"));
        assert!(body.contains(r#"register_sort_variant("shell[knuth]", &["sorts"], &[])"#));
    }

    #[test]
    fn missing_category_defaults_to_sort_in_the_domain_not_the_compiler() {
        let body = ArrayBackend::default()
            .entry_body(&EntryCtx { label: "x", category: None, adaptive: true, max_input: Some(32) })
            .unwrap();
        assert!(body.contains("category: avb_abi::Category::Sort"));
        assert!(body.contains("adaptive: true"));
        assert!(body.contains("max_input_size: Some(32)"));
    }

    #[test]
    fn non_sort_categories_select_their_driver_and_menu() {
        let be = ArrayBackend::default();
        let part = be
            .entry_body(&EntryCtx { label: "partition[lomuto]", category: Some("Partition"), adaptive: false, max_input: None })
            .unwrap();
        assert!(part.contains("run_partition_with_input"));
        assert!(part.contains(r#"&["partitions"]"#));

        let mrg = be
            .entry_body(&EntryCtx { label: "merge-op[two-finger]", category: Some("Merge"), adaptive: false, max_input: None })
            .unwrap();
        assert!(mrg.contains("run_merge_with_input"));
        assert!(mrg.contains(r#"&["merges"]"#));

        let rot = be
            .entry_body(&EntryCtx { label: "rotation[reversal]", category: Some("Rotation"), adaptive: false, max_input: None })
            .unwrap();
        assert!(rot.contains("run_rotation_with_input"));
        assert!(rot.contains(r#"&["rotations"]"#));
    }

    #[test]
    fn an_unmapped_category_is_the_backends_error_not_the_compilers() {
        let err = ArrayBackend::default()
            .entry_body(&EntryCtx { label: "x", category: Some("Galaxy"), adaptive: false, max_input: None })
            .unwrap_err();
        assert!(err.contains("no emit driver for category `Galaxy`"));
    }
}
