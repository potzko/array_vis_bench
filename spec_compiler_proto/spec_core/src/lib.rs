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
    fn registry_parses() {
        let r = reg();
        assert!(r.get("quick_sort").is_some());
        assert_eq!(r.providing("SinglePivot").len(), 3); // first, mid, med3
        assert_eq!(r.providing("DualPivot").len(), 1); // tukey
    }

    #[test]
    fn resolves_nested_tree() {
        let (alias, node) = parse_spec(
            "QuickLLMidIns32 = quick_sort< small_sort = insertion_sort<32> partition = LL_partition< pivot = middle_element > >",
        )
        .unwrap();
        assert_eq!(alias.as_deref(), Some("QuickLLMidIns32"));
        let r = resolve(&node, &reg()).unwrap();
        assert_eq!(
            r.type_expr,
            "QuickSort<LeftLeftPartition<MiddleElement>, InsertionSmallSort<32>>"
        );
        assert_eq!(r.label, "quick[LL<mid>/ins:32]");
    }

    #[test]
    fn fills_defaults_recursively() {
        let (_, node) = parse_spec("quick_sort< partition = LL_partition<> >").unwrap();
        let r = resolve(&node, &reg()).unwrap();
        // pivot -> first_element, small_sort -> no_small_sort
        assert_eq!(r.type_expr, "QuickSort<LeftLeftPartition<FirstElement>, NoSmallSort>");
        assert_eq!(r.label, "quick[LL<first>/none]");
    }

    #[test]
    fn arity_violation_is_rejected_by_the_engine() {
        // dual-pivot selector into a single-pivot partition: nesting + roles
        // catch it before rustc even sees it.
        let (_, node) = parse_spec("LL_partition< pivot = tukey_dual >").unwrap();
        let err = resolve(&node, &reg()).unwrap_err();
        assert!(err.contains("SinglePivot"), "got: {err}");
    }

    #[test]
    fn unknown_component_and_slot_error() {
        let (_, n1) = parse_spec("banana<>").unwrap();
        assert!(resolve(&n1, &reg()).unwrap_err().contains("unknown component"));
        let (_, n2) = parse_spec("LL_partition< pivt = first_element >").unwrap();
        assert!(resolve(&n2, &reg()).unwrap_err().contains("no slot named"));
    }

    #[test]
    fn enumerates_only_legal_combos() {
        let specs = enumerate(&reg(), "Sort", 5);
        // 7 partitions (LL×3 + LR×3 + dual×1) × 2 small sorts = 14
        assert_eq!(specs.len(), 14);
        let labels: Vec<String> = specs.iter().map(|s| resolve(s, &reg()).unwrap().label).collect();
        assert!(labels.iter().any(|l| l == "quick[dual<tukey>/none]"));
        assert!(labels.iter().any(|l| l == "quick[LL<mid>/ins:16]"));
        // never a single-pivot partition with the dual selector, nor vice-versa
        assert!(!labels.iter().any(|l| l.contains("LL<tukey>") || l.contains("dual<mid>")));
    }
}
