//! The shared engine: registry text → catalog, spec text → tree, tree + catalog
//! → resolved concrete type, and Rust emission. Pure std so both the proc-macro
//! and the generator binary reuse it.
//!
//! The pipeline stages each live in their own module (the boundaries are real,
//! but they share types and change together, so they're modules, not crates):
//!   - [`registry`]  stage 1 — load the text catalog (the specification contract)
//!   - [`spec`]      stage 2 — parse ONE spec tree OR a constraint-language query
//!   - [`resolve`]   stage 3 — name resolution + defaults + role/arity checks
//!   - [`emit`]      stage 4 — produce Rust source
//!   - [`enumerate`] stage 0 — naive flat enumeration (the contrast baseline)
//!   - [`solve`]     stage 0′ — the typed constraint solver: a query → a SET of ground trees (pinned / partial / full are one code path)
//!
//! rustc then performs stage 5 (full type checking + monomorphization).

pub mod emit;
pub mod emit_drivers;
pub mod enumerate;
pub mod registry;
pub mod resolve;
pub mod solve;
pub mod spec;

pub use emit::{emit_entries, emit_one, generate_table, EmitConfig};
pub use enumerate::enumerate;
pub use registry::{Component, Param, ParamKind, Registry};
pub use resolve::{resolve, Resolved};
pub use solve::{solve, SolveOutput};
pub use spec::{
    parse_query, parse_spec, Arg, Binding, QArg, QValue, Quant, Query, Refinement, SpecNode, Take,
    DEFAULT_DEPTH,
};

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
        // The NAIVE flat enumerator PRODUCES this bad combo (rustc is its only
        // gate); the constraint solver below never builds it. before/after:
        let labels: Vec<String> = quick.iter().map(|s| resolve(s, &reg()).unwrap().label).collect();
        assert!(labels.iter().any(|l| l == "quick[LL/ninther/none]"));
    }

    // ── the typed constraint language: pinned / partial / full, one evaluator ─

    fn ty_exprs(out: &solve::SolveOutput, reg: &Registry) -> Vec<String> {
        out.sorts.iter().map(|n| resolve(n, reg).unwrap().type_expr).collect()
    }
    fn labels(out: &solve::SolveOutput, reg: &Registry) -> Vec<String> {
        out.sorts.iter().map(|n| resolve(n, reg).unwrap().label).collect()
    }
    fn run(q: &str, reg: &Registry) -> solve::SolveOutput {
        solve(&parse_query(q).unwrap(), reg).unwrap()
    }

    #[test]
    fn same_evaluator_pinned_partial_full() {
        let reg = reg();

        // 0 holes → exactly 1 sort.
        let pinned = run(
            "let p: Pivot = first_element;
             let part: Partition[pivot = p] = LL_partition;
             let s: Sort = quick_sort(partition = part, pivot = p, small_sort = no_small_sort);",
            &reg,
        );
        assert_eq!(pinned.sorts.len(), 1);
        assert_eq!(
            ty_exprs(&pinned, &reg)[0],
            "QuickSort<LeftLeftPartition, FirstElement, NoSmallSort>"
        );

        // partial → a family: pivot pinned single ⇒ only the LL partition is
        // arity-compatible, crossed with the 3 small sorts.
        let partial = run(
            "let p: Pivot = first_element;
             let part: Partition[pivot = p] = .;
             let s: Sort = quick_sort(partition = part, pivot = p, small_sort = .);",
            &reg,
        );
        assert_eq!(partial.sorts.len(), 3);
        assert!(labels(&partial, &reg).iter().all(|l| l.starts_with("quick[LL/first/")));

        // all holes → many. Same solve(), zero code branches.
        let full = run("let s: Sort = .;", &reg);
        assert!(full.sorts.len() > partial.sorts.len());
        assert!(partial.sorts.len() > pinned.sorts.len());
    }

    // ── THE HEADLINE: a shared variable makes an arity mismatch unrepresentable ─
    #[test]
    fn shared_pivot_makes_arity_mismatch_unrepresentable() {
        let reg = reg();
        let out = run(
            "let p: Pivot = .;
             let part: Partition[pivot = p] = .;
             let s: Sort = quick_sort(partition = part, pivot = p, small_sort = .);",
            &reg,
        );
        // 7 pivots, each pinned to its arity-matching partition, × 3 small = 21.
        assert_eq!(out.sorts.len(), 21);

        let ls = labels(&out, &reg);
        // the dual selectors NEVER pair with the single-pivot LL partition…
        assert!(ls.iter().all(|l| !(l.starts_with("quick[LL/") && l.contains("ninther"))));
        assert!(ls.iter().all(|l| !(l.starts_with("quick[LL/") && l.contains("combined"))));
        // …and single pivots never pair with the dual partition.
        assert!(ls.iter().all(|l| !l.starts_with("quick[dual/first")));
        assert!(ls.iter().all(|l| !l.starts_with("quick[dual/mid")));
        // The exact combo the FLAT enumerator overproduces is simply absent —
        // not "produced then rejected by rustc", but never built.
        assert!(!ls.iter().any(|l| l == "quick[LL/ninther/none]"));
    }

    #[test]
    fn quantifier_counts_and_clamp() {
        let reg = reg();
        assert_eq!(run("let g: GapSequence = *;", &reg).sorts.len(), 3);
        assert_eq!(run("let g: GapSequence = .;", &reg).sorts.len(), 3);
        assert_eq!(run("let g: GapSequence = ?;", &reg).sorts.len(), 1);
        assert_eq!(run("let g: GapSequence = ?2@7;", &reg).sorts.len(), 2);

        // N > population clamps with a warning — never silent truncation.
        let clamped = run("let g: GapSequence = ?5@7;", &reg);
        assert_eq!(clamped.sorts.len(), 3);
        assert!(clamped.warnings.iter().any(|w| w.contains("clamped")));
    }

    #[test]
    fn n_of_yields_distinct_deduped_sorts() {
        let reg = reg();
        let three = run("3 of let sh: Sort = shell_sort(seq = .);", &reg);
        assert_eq!(three.sorts.len(), 3);
        let uniq: std::collections::HashSet<_> = ty_exprs(&three, &reg).into_iter().collect();
        assert_eq!(uniq.len(), 3, "the N sorts are distinct on canonical form");

        let over = run("100 of let sh: Sort = shell_sort(seq = .);", &reg);
        assert_eq!(over.sorts.len(), 3);
        assert!(over.warnings.iter().any(|w| w.contains("clamped")));
    }

    #[test]
    fn random_quantifier_is_seeded_and_reproducible() {
        let reg = reg();
        let a = labels(&run("let p: Pivot = ?3@42;", &reg), &reg);
        let b = labels(&run("let p: Pivot = ?3@42;", &reg), &reg);
        assert_eq!(a, b, "same seed → identical, reproducible build");
        assert_eq!(a.len(), 3);
        assert_eq!(run("let p: Pivot = ?3@99;", &reg).sorts.len(), 3);
    }

    #[test]
    fn recursive_grammar_terminates_under_depth() {
        let reg = reg();
        let count = |d: usize| run(&format!("depth {d}; let r: RecSort = .;"), &reg).sorts.len();
        // base, then one extra nesting level admitted per depth — sane, linear.
        assert_eq!(count(0), 1);
        assert_eq!(count(1), 2);
        assert_eq!(count(2), 3);
        assert_eq!(count(3), 4);
        // random sampling over the (bounded) recursive family stays distinct.
        assert_eq!(run("depth 3; let r: RecSort = ?2@1;", &reg).sorts.len(), 2);
    }

    #[test]
    fn consts_default_explicit_enumerated_and_shared() {
        let reg = reg();
        // default
        assert_eq!(ty_exprs(&run("let h: Sort = heap_sort();", &reg), &reg), ["HeapSort<2>"]);
        // explicit number
        assert_eq!(ty_exprs(&run("let h: Sort = heap_sort(arity = 4);", &reg), &reg), ["HeapSort<4>"]);
        // enumerate the declared "neat values" set (membership only)
        let mut all = ty_exprs(&run("let h: Sort = heap_sort(arity = *);", &reg), &reg);
        all.sort();
        assert_eq!(all, ["HeapSort<2>", "HeapSort<3>", "HeapSort<4>"]);
        // a shared const variable threaded to TWO slots: equal BY CONSTRUCTION
        // (pp == ee, never the mismatched pair) — structural equality on a number.
        assert_eq!(
            ty_exprs(
                &run(
                    "let flag: Flag = true;
                     let m: Sort = top_down_merge(small_sort = no_small_sort, ping_pong = flag, early_exit = flag);",
                    &reg,
                ),
                &reg,
            ),
            ["TopDownMergeSort<NoSmallSort, true, true>"]
        );
    }
}
