//! The shared engine: registry text → catalog, spec text → tree, tree + catalog
//! → resolved concrete type, and Rust emission. Pure std so both the proc-macro
//! and the generator binary reuse it.
//!
//! The pipeline stages each live in their own module (the boundaries are real,
//! but they share types and change together, so they're modules, not crates):
//!   - [`registry`]  stage 1 — load the text catalog (the specification contract)
//!   - [`spec`]      stage 2 — parse ONE spec tree
//!   - [`resolve`]   stage 3 — name resolution + defaults + role/arity checks
//!   - [`emit`]      stage 4 — produce Rust source
//!   - [`enumerate`] stage 0 (mode 2) — a program produces the spec trees
//! rustc then performs stage 5 (full type checking + monomorphization).

pub mod emit;
pub mod enumerate;
pub mod registry;
pub mod resolve;
pub mod spec;

pub use emit::{emit_one, generate_table};
pub use enumerate::enumerate;
pub use registry::{Component, Param, ParamKind, Registry};
pub use resolve::{resolve, Resolved};
pub use spec::{parse_spec, Arg, SpecNode};

#[cfg(test)]
mod tests {
    use super::*;

    const REG: &str = include_str!("../../registry.spec");

    fn reg() -> Registry {
        Registry::parse(REG).expect("registry parses")
    }

    #[test]
    fn registry_parses_with_imports() {
        let r = reg();
        let qs = r.get("quick_sort").unwrap();
        assert_eq!(qs.uses, vec!["crate::quick_sort_lib::quick_sort::QuickSort"]);
        assert_eq!(r.providing("Pivot").len(), 4); // first, mid, ninther, combined
    }

    #[test]
    fn type_plus_const_resolves_and_collects_uses() {
        // insertion is a TYPE slot (strategy) + a positional CONST (threshold).
        let (_, node) = parse_spec(
            "quick_sort< partition = LL_partition, pivot = middle_element, small_sort = insertion< strategy = binary, 32 > >",
        )
        .unwrap();
        let r = resolve(&node, &reg()).unwrap();
        assert_eq!(
            r.type_expr,
            "QuickSort<LeftLeftPartition, MiddleElement, InsertionSmallSort<BinaryInsertion, 32>>"
        );
        // imports unioned from every nested component
        assert!(r.uses.contains(&"crate::quick_sort_lib::quick_sort::QuickSort".to_string()));
        assert!(r.uses.contains(&"crate::small_sort_insertion::InsertionSmallSort".to_string()));
        assert!(r.uses.contains(&"crate::small_sort_insertion_strategy::BinaryInsertion".to_string()));
    }

    #[test]
    fn bool_consts_named_and_positional() {
        let (_, node) =
            parse_spec("top_down_merge< small_sort = no_small_sort, ping_pong = true, early_exit = false >").unwrap();
        let r = resolve(&node, &reg()).unwrap();
        assert_eq!(r.type_expr, "TopDownMergeSort<NoSmallSort, true, false>");
    }

    #[test]
    fn const_defaults_apply() {
        let (_, node) = parse_spec("insertion<>").unwrap();
        let r = resolve(&node, &reg()).unwrap();
        // strategy -> linear (default), N -> 16 (default)
        assert_eq!(r.type_expr, "InsertionSmallSort<LinearInsertion, 16>");
    }

    // ── THE HIDDEN ISSUE: flat layout cannot pre-check cross-slot arity ───────
    #[test]
    fn flat_quicksort_accepts_arity_mismatch_at_registry_level() {
        // single-pivot partition + DUAL selector. Each slot is individually
        // role-valid (LL provides Partition, ninther provides Pivot), so the
        // per-slot role check PASSES — the registry emits a type that only
        // rustc rejects (via `V: PivotInput<Arity = P::Arity>`). The nested
        // layout would have made arity local; the flat layout cannot.
        let (_, node) = parse_spec(
            "quick_sort< partition = LL_partition, pivot = ninther_dual, small_sort = no_small_sort >",
        )
        .unwrap();
        let r = resolve(&node, &reg()).expect("registry accepts it (the gap)");
        assert_eq!(r.type_expr, "QuickSort<LeftLeftPartition, NintherDualPivot, NoSmallSort>");
    }

    #[test]
    fn unknown_component_and_slot_error() {
        let (_, n1) = parse_spec("banana<>").unwrap();
        assert!(resolve(&n1, &reg()).unwrap_err().contains("unknown component"));
        let (_, n2) = parse_spec("LL_partition< pivt = first_element >").unwrap();
        assert!(resolve(&n2, &reg()).unwrap_err().contains("no slot named"));
    }

    #[test]
    fn enumeration_overproduces_quicksort_arity_combos() {
        let specs = enumerate(&reg(), "Sort", 5);
        let quick: Vec<_> = specs.iter().filter(|s| s.name == "quick_sort").collect();
        // partitions(2) × pivots(7: first, mid, ninther, combined{first,mid}×{first,mid}=4) × small(3) = 42
        assert_eq!(quick.len(), 42);
        // ...and some are arity-illegal — e.g. LL partition with the dual ninther.
        let labels: Vec<String> = quick.iter().map(|s| resolve(s, &reg()).unwrap().label).collect();
        assert!(labels.iter().any(|l| l == "quick[LL/ninther/none]"));
    }
}
