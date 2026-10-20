//! The shared engine: registry text → catalog, spec text → tree, tree + catalog
//! → resolved concrete type, and Rust emission. Pure std so both the proc-macro
//! and the generator binary reuse it.
//!
//! The pipeline stages each live in their own module (the boundaries are real,
//! but they share types and change together, so they're modules, not crates):
//!   - [`registry`]  stage 1 — load the text catalog (the specification contract)
//!   - [`spec`]      stage 2 — parse ONE spec tree OR a constraint-language query
//!   - [`resolve`]   stage 3, name resolution + role/arity checks
//!   - [`emit`]      stage 4 — produce Rust source (structural scaffold only)
//!   - [`backend`]   the domain seam — `emit` defers each entry's registration body to an [`EmitBackend`]
//!   - [`enumerate`] stage 0 — naive flat enumeration (the contrast baseline)
//!   - [`solve`]     stage 0′ — the typed constraint solver: a query → a SET of ground trees (pinned / partial / full are one code path)
//!
//! rustc then performs stage 5 (full type checking + monomorphization).

/// The pest-based AVBS *query* language frontend (parse-only, shape lock).
/// Optional so the default engine build stays dependency-free.
#[cfg(feature = "pest-frontend")]
pub mod avbs;
pub mod backend;
pub mod cardinality;
pub mod emit;
pub mod enumerate;
pub mod registry;
pub mod resolve;
pub mod solve;
pub mod spec;

pub use backend::{EmitBackend, EntryCtx, Wrapper};
pub use cardinality::{card_role, check_size, estimate, Card, SizeEstimate, DEFAULT_MAX_GROUND};
pub use emit::{emit_entries, emit_one, generate_table};
pub use enumerate::enumerate;
pub use registry::{Component, Param, ParamKind, Registry};
pub use resolve::{resolve, AxisNode, Resolved};
pub use solve::{solve, SolveOutput};
pub use spec::{
    parse_query, parse_spec, Arg, Binding, QArg, QValue, Quant, Query, Refinement, SpecNode, Take,
    DEFAULT_DEPTH,
};

#[cfg(test)]
mod tests {
    use super::*;

    // Engine self-test fixture: this crate's own example catalog.
    const REG: &str = include_str!("../registry.spec");

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
    fn nested_slots_resolve_and_collect_uses() {
        // The partition composite nests its scheme and pivot; the small sort nests
        // its strategy.
        let (_, node) = parse_spec(
            "quick_sort< partition = pk_single< scheme = LL_partition, pivot = middle_element >, \
                         small_sort = insertion< strategy = binary > >",
        )
        .unwrap();
        let r = resolve(&node, &reg()).unwrap();
        assert_eq!(
            r.type_expr,
            "QuickSort<Pivoted<LeftLeftPartition, MiddleElement>, InsertionSmallSort<BinaryInsertion, 16>>"
        );
        // imports unioned from every nested component
        assert!(r.uses.contains(&"crate::quick_sort_lib::quick_sort::QuickSort".to_string()));
        assert!(r.uses.contains(&"crate::quick_sort_lib::Pivoted".to_string()));
        assert!(r.uses.contains(&"crate::small_sort_insertion::InsertionSmallSort".to_string()));
        assert!(r.uses.contains(&"crate::small_sort_insertion_strategy::BinaryInsertion".to_string()));
    }

    #[test]
    fn bool_consts_named_and_positional() {
        let named =
            parse_spec("top_down_merge< small_sort = no_small_sort, ping_pong = true, early_exit = false >").unwrap().1;
        let positional = parse_spec("top_down_merge< small_sort = no_small_sort, true, false >").unwrap().1;
        for node in [named, positional] {
            assert_eq!(resolve(&node, &reg()).unwrap().type_expr, "TopDownMergeSort<NoSmallSort, true, false>");
        }
    }

    #[test]
    fn omitted_slot_and_const_are_errors() {
        let (_, no_strategy) = parse_spec("insertion<>").unwrap();
        let err = resolve(&no_strategy, &reg()).unwrap_err();
        assert!(err.contains("requires slot `strategy` (role InsertionStrategy)"), "got: {err}");
        let (_, no_arity) = parse_spec("heap_sort<>").unwrap();
        let err = resolve(&no_arity, &reg()).unwrap_err();
        assert!(err.contains("needs a value for const `arity`"), "got: {err}");
    }

    #[test]
    fn a_hole_over_a_literal_only_const_is_an_error() {
        let reg = Registry::parse("component c\n  type C<{n}>\n  provides S\n  const n\nend\n").unwrap();
        let err = solve(&parse_query("let s: S = c(n = .);").unwrap(), &reg).unwrap_err();
        assert!(err.contains("literal only"), "got: {err}");
    }

    #[test]
    fn solver_reports_a_query_that_omits_a_slot() {
        let err = solve(&parse_query("let s: Sort = quick_sort(partition = .);").unwrap(), &reg()).unwrap_err();
        assert!(err.contains("requires slot `small_sort` (role SmallSort)"), "got: {err}");
    }

    #[test]
    fn unknown_component_and_slot_error() {
        let (_, n1) = parse_spec("banana<>").unwrap();
        assert!(resolve(&n1, &reg()).unwrap_err().contains("unknown component"));
        let (_, n2) = parse_spec("pk_single< pivt = first_element >").unwrap();
        assert!(resolve(&n2, &reg()).unwrap_err().contains("no slot named"));
    }

    #[test]
    fn enumeration_builds_only_arity_legal_quicksort_combos() {
        let specs = enumerate(&reg(), "Sort", 5);
        let quick: Vec<_> = specs.iter().filter(|s| s.name == "quick_sort").collect();
        // partitions(7: single composite over first, mid; dual composite over ninther
        // and combined{first,mid}x{first,mid}=4) x small(3) = 21
        assert_eq!(quick.len(), 21);
        // The arity is baked into the composite, so even the naive enumerator never
        // offers a dual selector to the single composite or the reverse.
        let labels: Vec<String> = quick.iter().map(|s| resolve(s, &reg()).unwrap().label).collect();
        let single: Vec<_> = labels.iter().filter(|l| l.starts_with("quick[single<")).collect();
        let dual: Vec<_> = labels.iter().filter(|l| l.starts_with("quick[dual<")).collect();
        assert_eq!((single.len(), dual.len()), (6, 15));
        assert!(single.iter().all(|l| !l.contains("ninther") && !l.contains("combined")));
        assert!(dual.iter().all(|l| l.contains("ninther") || l.contains("combined")));
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
             let part: Partition[pivot = p] = pk_single(scheme = LL_partition, pivot = p);
             let s: Sort = quick_sort(partition = part, small_sort = no_small_sort);",
            &reg,
        );
        assert_eq!(pinned.sorts.len(), 1);
        assert_eq!(
            ty_exprs(&pinned, &reg)[0],
            "QuickSort<Pivoted<LeftLeftPartition, FirstElement>, NoSmallSort>"
        );

        // partial, a family: a pivot pinned single lets only the single composite
        // pass the refinement, its 2 partitions crossed with the 3 small sorts.
        let partial = run(
            "let p: Pivot = first_element;
             let part: Partition[pivot = p] = .;
             let s: Sort = quick_sort(partition = part, small_sort = .);",
            &reg,
        );
        assert_eq!(partial.sorts.len(), 6);
        assert!(labels(&partial, &reg).iter().all(|l| l.starts_with("quick[single<LL, ")));

        // all holes → many. Same solve(), zero code branches.
        let full = run("let s: Sort = quick_sort(partition = ., small_sort = .);", &reg);
        assert!(full.sorts.len() > partial.sorts.len());
        assert!(partial.sorts.len() > pinned.sorts.len());
    }

    // A refinement admits only the partitions whose pivot slot the shared
    // variable's value can fill.
    #[test]
    fn refinement_keeps_only_arity_matching_partitions() {
        let reg = reg();
        let with_pivot = |pivot: &str| {
            run(
                &format!(
                    "let p: Pivot = {pivot};
                     let part: Partition[pivot = p] = .;
                     let s: Sort = quick_sort(partition = part, small_sort = .);"
                ),
                &reg,
            )
        };
        // a single selector keeps the single composite: 2 partitions x 3 small = 6.
        let single = labels(&with_pivot("first_element"), &reg);
        assert_eq!(single.len(), 6);
        assert!(single.iter().all(|l| l.starts_with("quick[single<")));
        // a dual selector keeps the dual composite: 5 partitions x 3 small = 15.
        let dual = labels(&with_pivot("ninther_dual"), &reg);
        assert_eq!(dual.len(), 15);
        assert!(dual.iter().all(|l| l.starts_with("quick[dual<")));
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
    fn consts_explicit_enumerated_and_shared() {
        let reg = reg();
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
