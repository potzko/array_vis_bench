//! Stage 0″ — STATIC SIZE ANALYSIS. Before `solve` materializes anything, this
//! pass computes an *upper bound* on how much the query will produce, so a
//! compile that would never terminate (an unbounded recursive role) or take
//! forever (an astronomically large cross-product) is rejected with a clear
//! error BEFORE the expensive `enumerate`/cross-product loop runs.
//!
//! The idea (the user's): give every construct a **cardinality** and propagate
//! it compositionally — exactly like a type, but the "type" is *how many ground
//! values it stands for*. A role's cardinality is the size of its (depth-bounded)
//! population; a union is the **sum** of its alternatives; a set-difference's
//! cost is its **base** (subtraction can only shrink the result, but the base is
//! what gets materialized); a component application is the **product** of its
//! argument cardinalities; a captured-variable reference is **1** (it is coupled,
//! not multiplied). The whole query is the **product** across its bindings (the
//! env cross-product `solve` walks).
//!
//! Two numbers come out of it:
//!  - `ground` — an upper bound on the number of ground sorts emitted.
//!  - `peak`   — an upper bound on the largest population materialized at any
//!    single step (the cross-product, or one huge hole). `peak` is what actually
//!    drives compile time / memory, so it is what the budget guards.
//!
//! Faithfulness: [`card_role`] reproduces `enumerate(reg, role, depth).len()`
//! exactly (same recurrence, counting instead of building), so the bound is never
//! an under-count of the work. Where capture-coupling or dedup collapse the real
//! result it can over-count — but a captured ref contributes 1, so the common
//! `combined<a = _ as x, b = x>` diagonal is counted tightly, not as the square.

use std::collections::{HashMap, HashSet};

use crate::registry::{ParamKind, Registry};
use crate::spec::{Binding, QArg, QValue, Quant, Query};

/// A saturating cardinality: a finite count, or `Huge` once a saturating add /
/// multiply would overflow `u128` (i.e. "too large to bother counting — already
/// far past any sane build"). Saturation keeps the analysis itself cheap and
/// total: a super-exponential recursive role collapses to `Huge` in a few steps
/// instead of computing a 10⁴⁵-digit number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Card {
    Finite(u128),
    Huge,
}

impl Card {
    pub fn is_huge(self) -> bool {
        matches!(self, Card::Huge)
    }

    /// Does this exceed a finite budget? `Huge` always does.
    pub fn exceeds(self, budget: u128) -> bool {
        match self {
            Card::Huge => true,
            Card::Finite(n) => n > budget,
        }
    }

    fn add(self, o: Card) -> Card {
        match (self, o) {
            (Card::Finite(a), Card::Finite(b)) => {
                a.checked_add(b).map(Card::Finite).unwrap_or(Card::Huge)
            }
            _ => Card::Huge,
        }
    }

    fn mul(self, o: Card) -> Card {
        match (self, o) {
            (Card::Finite(a), Card::Finite(b)) => {
                a.checked_mul(b).map(Card::Finite).unwrap_or(Card::Huge)
            }
            _ => Card::Huge,
        }
    }

    fn max(self, o: Card) -> Card {
        match (self, o) {
            (Card::Huge, _) | (_, Card::Huge) => Card::Huge,
            (Card::Finite(a), Card::Finite(b)) => Card::Finite(a.max(b)),
        }
    }

    /// Cap a population at a sample size `n` (`min`).
    fn cap(self, n: u128) -> Card {
        match self {
            Card::Huge => Card::Finite(n),
            Card::Finite(m) => Card::Finite(m.min(n)),
        }
    }
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Card::Finite(n) => write!(f, "{n}"),
            Card::Huge => write!(f, "astronomically large (overflowed u128)"),
        }
    }
}

/// The result of the static size analysis over a whole [`Query`].
#[derive(Debug, Clone)]
pub struct SizeEstimate {
    /// Upper bound on the number of ground sorts the query yields (after `take`).
    pub ground: Card,
    /// Upper bound on the most work any single binding incurs: its population
    /// times the number of environments it is evaluated against (the prefix
    /// cross-product). This dominates compile time + memory, so it is the figure
    /// the budget guards — and the full env cross-product (`ground`, pre-take) is
    /// always ≤ it.
    pub peak: Card,
    /// The binding contributing the largest peak (name, its peak) — for messages.
    pub worst_binding: Option<(String, Card)>,
    /// Catalog roles that are recursive (reach themselves through a type-slot):
    /// their cardinality is driven entirely by `depth`, so an unbounded depth
    /// here is the "never terminates" hazard. Sorted, for stable diagnostics.
    pub recursive_roles: Vec<String>,
}

/// Default ceiling on `peak` before `solve` refuses to run. A million ground
/// candidates is "large but finishes"; beyond it a build is almost certainly a
/// mistake (an unbounded recursion or an unintended full cross-product). Raise it
/// per-call with [`check_size`] when a big sweep is genuinely intended.
pub const DEFAULT_MAX_GROUND: u128 = 1_000_000;

/// Forward cardinality (`out`, what flows to the parent) + peak materialization
/// (`peak`, the largest set built anywhere in this subtree).
#[derive(Debug, Clone, Copy)]
struct Est {
    out: Card,
    peak: Card,
}

impl Est {
    fn one() -> Est {
        Est { out: Card::Finite(1), peak: Card::Finite(1) }
    }
}

/// Memo for `card_role(role, depth)` and `card_component(name, depth)`.
#[derive(Default)]
struct Memo {
    role: HashMap<(String, usize), Card>,
    comp: HashMap<(String, usize), Card>,
}

/// The number of ground trees `enumerate(reg, role, depth)` would produce —
/// computed by the same recurrence, counting instead of materializing. This is
/// the cardinality of an exhaustive hole of `role`.
pub fn card_role(reg: &Registry, role: &str, depth: usize) -> Card {
    let mut memo = Memo::default();
    card_role_memo(reg, role, depth, &mut memo)
}

fn card_role_memo(reg: &Registry, role: &str, depth: usize, memo: &mut Memo) -> Card {
    let key = (role.to_string(), depth);
    if let Some(c) = memo.role.get(&key) {
        return *c;
    }
    // Guard against an over-deep memo while a sibling is in flight: seed with a
    // placeholder so a cyclic role at the SAME depth can't loop. (Depth strictly
    // decreases on every recursive edge, so this never actually fires — it is a
    // belt-and-braces total-ity guarantee.)
    memo.role.insert(key.clone(), Card::Finite(0));
    let mut total = Card::Finite(0);
    for comp in reg.providing(role) {
        total = total.add(card_component_memo(reg, &comp.name, depth, memo));
    }
    memo.role.insert(key, total);
    total
}

fn card_component_memo(reg: &Registry, name: &str, depth: usize, memo: &mut Memo) -> Card {
    let key = (name.to_string(), depth);
    if let Some(c) = memo.comp.get(&key) {
        return *c;
    }
    let comp = match reg.get(name) {
        Some(c) => c,
        None => return Card::Finite(0),
    };
    let type_roles: Vec<String> = comp
        .params
        .iter()
        .filter_map(|p| match &p.kind {
            ParamKind::Type { role, .. } => Some(role.clone()),
            _ => None,
        })
        .collect();

    let card = if type_roles.is_empty() {
        Card::Finite(1) // leaf (or const-only): one structural variant
    } else if depth == 0 {
        Card::Finite(0) // can't nest further on this path
    } else {
        // cartesian product over the type slots, each filled at depth-1.
        let mut prod = Card::Finite(1);
        for r in &type_roles {
            prod = prod.mul(card_role_memo(reg, r, depth - 1, memo));
        }
        prod
    };
    memo.comp.insert(key, card);
    card
}

/// Estimate a whole query's output + peak materialization, plus the catalog's
/// recursive roles. Never runs `enumerate`; purely structural.
pub fn estimate(query: &Query, reg: &Registry) -> SizeEstimate {
    let mut memo = Memo::default();
    // `prefix` = product of the `out`s of bindings already processed — i.e. the
    // number of environments `solve` carries INTO the next binding. `solve` calls
    // `eval_binding` once per environment, so binding i's population is
    // materialized `prefix` times: its true cost is `prefix * est.peak`, not just
    // `est.peak`. (A sample-then-multiply binding — a huge hole sampled to 1 after
    // a wide prefix — would otherwise be wildly under-counted.)
    let mut prefix = Card::Finite(1);
    let mut peak = Card::Finite(1);
    let mut worst: Option<(String, Card)> = None;

    for b in &query.bindings {
        let est = est_binding(b, reg, query.depth, &mut memo);
        let cost = prefix.mul(est.peak);
        peak = peak.max(cost);
        if worst.as_ref().map(|(_, c)| cost.exceeds(card_u128(*c))).unwrap_or(true) {
            worst = Some((b.name.clone(), cost));
        }
        prefix = prefix.mul(est.out);
    }

    // After the loop `prefix` is the full env cross-product = the ground count.
    let mut ground = prefix;
    if let Some(take) = query.take {
        ground = ground.cap(take.n as u128);
    }

    SizeEstimate { ground, peak, worst_binding: worst, recursive_roles: recursive_roles(reg) }
}

/// Run the analysis and reject the query if its `peak` exceeds `budget`. The
/// `solve` entry point calls this with [`DEFAULT_MAX_GROUND`] so a runaway query
/// fails fast, before any population is built.
pub fn check_size(query: &Query, reg: &Registry, budget: u128) -> Result<SizeEstimate, String> {
    let est = estimate(query, reg);
    if est.peak.exceeds(budget) {
        let where_ = match &est.worst_binding {
            Some((name, c)) => format!(" (binding `{name}` alone reaches {c})"),
            None => String::new(),
        };
        let rec = if est.recursive_roles.is_empty() {
            String::new()
        } else {
            format!(
                " The catalog has recursive role(s) [{}] whose size grows with \
                 `depth` — lower `depth`, sample with `?N`, or narrow with \
                 `where (…)` / `_ - {{…}}`.",
                est.recursive_roles.join(", ")
            )
        };
        return Err(format!(
            "query is too large to compile: estimated peak materialization of \
             {peak} trees exceeds the budget of {budget}{where_}.{rec}",
            peak = est.peak,
        ));
    }
    Ok(est)
}

// ── per-binding / per-value estimation ───────────────────────────────────────

fn est_binding(b: &Binding, reg: &Registry, depth: usize, memo: &mut Memo) -> Est {
    // A const binding is a single value.
    if let QValue::Const(_) = &b.value {
        return Est::one();
    }
    est_value(&b.value, &b.role, reg, depth, memo)
}

/// Estimate a value in TYPE context against `role`.
fn est_value(value: &QValue, role: &str, reg: &Registry, depth: usize, memo: &mut Memo) -> Est {
    match value {
        // A hole ranges over the whole (depth-bounded) role population; a sample
        // caps the OUTPUT but still materializes the full population first.
        QValue::Hole(q) => {
            let pop = card_role_memo(reg, role, depth, memo);
            Est { out: cap_quant(pop, *q), peak: pop }
        }
        // A reference (earlier binding / nullary component / captured var): one
        // value per environment — coupled, never multiplied.
        QValue::Ident(_) | QValue::Const(_) => Est::one(),
        QValue::Capture { inner, .. } => est_value(inner, role, reg, depth, memo),
        // A union: population = sum of the alternatives, then sampled.
        QValue::Where { quant, alts } => {
            let mut sum = Card::Finite(0);
            let mut peak = Card::Finite(0);
            for a in alts {
                let e = est_value(a, role, reg, depth, memo);
                sum = sum.add(e.out);
                peak = peak.max(e.peak);
            }
            Est { out: cap_quant(sum, *quant), peak: peak.max(sum) }
        }
        // A set literal (a subtrahend): the sum of its members' populations.
        QValue::Set(members) => {
            let mut sum = Card::Finite(0);
            let mut peak = Card::Finite(0);
            for m in members {
                let e = est_value(m, role, reg, depth, memo);
                sum = sum.add(e.out);
                peak = peak.max(e.peak);
            }
            Est { out: sum, peak: peak.max(sum) }
        }
        // Set difference: the COST is the base (it is fully materialized, then
        // filtered) — subtraction only shrinks the result, never grows the work.
        // So the output bound is the base's output; the peak is the base's peak
        // plus whatever the subtrahends materialize.
        QValue::Diff { base, subtrahends } => {
            let b = est_value(base, role, reg, depth, memo);
            let mut peak = b.peak;
            for s in subtrahends {
                peak = peak.max(est_value(s, role, reg, depth, memo).peak);
            }
            Est { out: b.out, peak }
        }
        // A component application: the product over its arguments. Type args
        // estimate against the param's role; const args range over the param's
        // declared value set (or 1 for a literal). Captures make some args
        // resolve to 1, so a coupled cross-product is counted tightly.
        QValue::App { name, args } => est_app(name, args, reg, depth, memo),
    }
}

fn est_app(
    name: &str,
    args: &[QArg],
    reg: &Registry,
    depth: usize,
    memo: &mut Memo,
) -> Est {
    let comp = match reg.get(name) {
        Some(c) => c,
        None => return Est::one(), // unknown head → treat as a single ref
    };
    let mut prod = Card::Finite(1);
    let mut peak = Card::Finite(1);
    for a in args {
        let e = match a {
            QArg::Named { name: pname, value } => match comp.param(pname) {
                Some(p) => match &p.kind {
                    ParamKind::Type { role, .. } => est_value(value, role, reg, depth, memo),
                    ParamKind::Const { values, .. } => est_const(value, values),
                    ParamKind::Project { .. } => Est::one(),
                },
                None => est_value(value, "", reg, depth, memo),
            },
            // Positional consts (legacy `parse_query`): a literal is 1.
            QArg::Pos(value) => est_const(value, &[]),
        };
        prod = prod.mul(e.out);
        peak = peak.max(e.peak);
    }
    // The application itself materializes the running cross-product of its args.
    Est { out: prod, peak: peak.max(prod) }
}

/// Estimate a value in CONST context: a hole ranges over the declared "neat
/// values" (or degenerates to one default); a literal / ref is a single value.
fn est_const(value: &QValue, values: &[String]) -> Est {
    match value {
        QValue::Hole(q) => {
            let pop = Card::Finite(values.len().max(1) as u128);
            Est { out: cap_quant(pop, *q), peak: pop }
        }
        _ => Est::one(),
    }
}

/// Apply a hole's quantifier to a population size (sampling caps it).
fn cap_quant(pop: Card, q: Quant) -> Card {
    match q {
        Quant::Exhaustive => pop,
        Quant::One { .. } => pop.cap(1),
        Quant::N { n, .. } => pop.cap(n as u128),
    }
}

fn card_u128(c: Card) -> u128 {
    match c {
        Card::Finite(n) => n,
        Card::Huge => u128::MAX,
    }
}

// ── recursive-role detection (the "loop" detector) ───────────────────────────

/// Every role that can reach itself through a chain of type-slots — i.e. a cycle
/// in the role-provider graph. These are the roles whose population is unbounded
/// in principle and only finite because of the `depth` cap. Sorted by name.
pub fn recursive_roles(reg: &Registry) -> Vec<String> {
    // Build edges role → roles of the type-slots of any component providing it.
    let mut edges: HashMap<String, HashSet<String>> = HashMap::new();
    let mut roles: HashSet<String> = HashSet::new();
    for comp in reg.iter() {
        for r in &comp.provides {
            roles.insert(r.clone());
            let entry = edges.entry(r.clone()).or_default();
            for p in &comp.params {
                if let ParamKind::Type { role, .. } = &p.kind {
                    entry.insert(role.clone());
                }
            }
        }
    }
    let mut out: Vec<String> = roles
        .iter()
        .filter(|r| reaches_self(r, &edges))
        .cloned()
        .collect();
    out.sort();
    out
}

/// DFS: can `start` reach itself by following slot-role edges?
fn reaches_self(start: &str, edges: &HashMap<String, HashSet<String>>) -> bool {
    let mut stack: Vec<&str> = edges.get(start).into_iter().flatten().map(String::as_str).collect();
    let mut seen: HashSet<&str> = HashSet::new();
    while let Some(r) = stack.pop() {
        if r == start {
            return true;
        }
        if !seen.insert(r) {
            continue;
        }
        if let Some(next) = edges.get(r) {
            stack.extend(next.iter().map(String::as_str));
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enumerate::enumerate;
    use crate::spec::parse_query;

    const REG: &str = include_str!("../registry.spec");

    fn fixture() -> Registry {
        Registry::parse(REG).expect("registry parses")
    }

    fn fin(c: Card) -> u128 {
        match c {
            Card::Finite(n) => n,
            Card::Huge => panic!("expected Finite, got Huge"),
        }
    }

    // ── card_role is faithful to enumerate (the load-bearing invariant) ────────

    #[test]
    fn card_role_matches_enumerate_count() {
        let reg = fixture();
        for role in ["Sort", "Pivot", "PivotSingle", "Partition", "GapSequence", "SmallSort"] {
            for depth in 0..6usize {
                let counted = card_role(&reg, role, depth);
                let enumerated = enumerate(&reg, role, depth).len() as u128;
                assert_eq!(
                    fin(counted),
                    enumerated,
                    "role {role} @ depth {depth}: counted {counted} vs enumerated {enumerated}"
                );
            }
        }
    }

    // ── per-construct estimates ────────────────────────────────────────────────

    fn est_of(src: &str, reg: &Registry) -> SizeEstimate {
        estimate(&parse_query(src).unwrap(), reg)
    }

    #[test]
    fn shell_hole_estimate() {
        let reg = fixture();
        // fixture has 3 gap sequences.
        let e = est_of("s: Sort = shell_sort(seq = .);", &reg);
        assert_eq!(fin(e.ground), 3);
        assert_eq!(fin(e.peak), 3);
    }

    #[test]
    fn product_rule_over_an_application() {
        let reg = fixture();
        // uncoupled cross-product: 2 single pivots × 2 = 4 (legacy `parse_query`
        // syntax: component name + parens). Capture-coupling (→ 2) is exercised
        // via the avbs frontend tests.
        let cross = est_of("p: Pivot = combined(a = ., b = .);", &reg);
        assert_eq!(fin(cross.ground), 4);
    }

    #[test]
    fn sample_caps_output_but_not_peak() {
        let reg = fixture();
        let e = est_of("p: Pivot = ?2@1;", &reg);
        assert_eq!(fin(e.ground), 2, "output capped to the sample size");
        assert_eq!(fin(e.peak), 7, "but the full 7-pivot population is materialized first");
    }

    #[test]
    fn multi_binding_peak_accounts_for_repeated_materialization() {
        let reg = fixture();
        // b1 ranges over 7 pivots; b2 materializes the 3-sequence population once
        // PER pivot environment (solve calls eval_binding per env), so the real
        // peak is 7 × 3 = 21 — not just max(7, 3). ground stays 7 × 1 = 7.
        let q = parse_query("let p: Pivot = .; let g: GapSequence = ?1@0;").unwrap();
        let e = estimate(&q, &reg);
        assert_eq!(fin(e.ground), 7, "7 pivots × 1 sampled sequence");
        assert_eq!(fin(e.peak), 21, "the sequence population is built once per pivot env");
        assert_eq!(e.worst_binding.as_ref().map(|(n, _)| n.as_str()), Some("g"));
    }

    #[test]
    fn take_caps_ground_but_not_peak() {
        let reg = fixture();
        let e = est_of("3 of let s: Sort = quick_sort(partition = ., pivot = ., small_sort = .);", &reg);
        assert_eq!(fin(e.ground), 3, "`N of` caps the emitted count");
        // peak is the full pre-take cross-product (42 quick combos), unprotected.
        assert_eq!(fin(e.peak), 42);
    }

    #[test]
    fn set_diff_cost_is_the_base_population() {
        let reg = fixture();
        // The base (all 7 pivots) is the materialized cost; subtraction can only
        // shrink the output, so the bound stays at the base size (upper bound).
        let q = Query {
            depth: crate::spec::DEFAULT_DEPTH,
            take: None,
            bindings: vec![Binding {
                name: "p".into(),
                role: "Pivot".into(),
                refinements: vec![],
                value: QValue::Diff {
                    base: Box::new(QValue::Hole(Quant::Exhaustive)),
                    subtrahends: vec![QValue::Set(vec![QValue::Ident("middle_element".into())])],
                },
            }],
        };
        let e = estimate(&q, &reg);
        assert_eq!(fin(e.peak), 7);
    }

    #[test]
    fn where_union_is_the_sum_of_alternatives() {
        let reg = fixture();
        // union population = first(1) + combined a×b(4) = 5.
        let q = Query {
            depth: crate::spec::DEFAULT_DEPTH,
            take: None,
            bindings: vec![Binding {
                name: "p".into(),
                role: "Pivot".into(),
                refinements: vec![],
                value: QValue::Where {
                    quant: Quant::Exhaustive,
                    alts: vec![
                        QValue::Ident("first_element".into()),
                        QValue::App {
                            name: "combined".into(),
                            args: vec![
                                QArg::Named { name: "a".into(), value: QValue::Hole(Quant::Exhaustive) },
                                QArg::Named { name: "b".into(), value: QValue::Hole(Quant::Exhaustive) },
                            ],
                        },
                    ],
                },
            }],
        };
        let e = estimate(&q, &reg);
        assert_eq!(fin(e.ground), 5);
    }

    // ── recursion detection ────────────────────────────────────────────────────

    #[test]
    fn detects_recursive_roles() {
        let reg = fixture();
        // registry.spec's RecSort is self-recursive (recursive_sort has a RecSort slot).
        assert!(recursive_roles(&reg).contains(&"RecSort".to_string()));
        // Sort/Pivot/etc. are not.
        assert!(!recursive_roles(&reg).contains(&"Pivot".to_string()));
    }

    #[test]
    fn recursive_role_count_grows_with_depth_but_stays_finite() {
        let reg = fixture();
        // matches the existing solve test: 1, 2, 3, 4 at depths 0..3.
        assert_eq!(fin(card_role(&reg, "RecSort", 0)), 1);
        assert_eq!(fin(card_role(&reg, "RecSort", 3)), 4);
    }

    // ── the blow-up catalog: a binary tree explodes super-exponentially ─────────

    const TREE: &str = "\
component node
  type Node<{l}, {r}>
  provides Tree
  slot l Tree
  slot r Tree
end
component leaf
  type Leaf
  provides Tree
end
";

    fn tree() -> Registry {
        Registry::parse(TREE).unwrap()
    }

    #[test]
    fn tree_cardinality_is_super_exponential() {
        let reg = tree();
        // T(d) = 1 + T(d-1)^2 : 1, 2, 5, 26, 677, 458330, ...
        assert_eq!(fin(card_role(&reg, "Tree", 0)), 1);
        assert_eq!(fin(card_role(&reg, "Tree", 1)), 2);
        assert_eq!(fin(card_role(&reg, "Tree", 2)), 5);
        assert_eq!(fin(card_role(&reg, "Tree", 3)), 26);
        assert_eq!(fin(card_role(&reg, "Tree", 4)), 677);
        assert_eq!(fin(card_role(&reg, "Tree", 5)), 458_330);
        assert!(recursive_roles(&reg).contains(&"Tree".to_string()));
    }

    #[test]
    fn deep_recursion_saturates_to_huge_cheaply() {
        let reg = tree();
        // By depth ~8 the count overflows u128 — the analysis returns Huge in a
        // handful of steps instead of computing a 45-digit number (and never
        // hangs, which is the whole point).
        assert!(card_role(&reg, "Tree", 12).is_huge());
    }

    #[test]
    fn solve_warns_when_approaching_the_budget() {
        let reg = fixture();
        let q = parse_query("let p: Pivot = .;").unwrap();
        // 7 pivots, budget 20 → 20/4 = 5, and 7 > 5: under budget but flagged.
        let out = crate::solve::solve_within(&q, &reg, 20).unwrap();
        assert_eq!(out.sorts.len(), 7);
        assert!(
            out.warnings.iter().any(|w| w.contains("large build")),
            "expected a heads-up warning, got: {:?}",
            out.warnings
        );
        // Under the default (million) budget there is no such warning.
        let quiet = crate::solve::solve(&q, &reg).unwrap();
        assert!(!quiet.warnings.iter().any(|w| w.contains("large build")));
    }

    #[test]
    fn check_size_passes_under_budget_and_rejects_over() {
        let reg = tree();
        let q = |d: usize| Query {
            depth: d,
            take: None,
            bindings: vec![Binding {
                name: "t".into(),
                role: "Tree".into(),
                refinements: vec![],
                value: QValue::Hole(Quant::Exhaustive),
            }],
        };
        // depth 5 → 458_330, under the default million → OK.
        let ok = check_size(&q(5), &reg, DEFAULT_MAX_GROUND).unwrap();
        assert_eq!(fin(ok.ground), 458_330);

        // depth 6 → ~2.1e11, over budget → a clear, actionable error.
        let err = check_size(&q(6), &reg, DEFAULT_MAX_GROUND).unwrap_err();
        assert!(err.contains("too large"), "got: {err}");
        assert!(err.contains("Tree"), "names the recursive role: {err}");

        // `N of` does NOT save it — the peak cross-product is still built.
        let mut sampled = q(6);
        sampled.take = Some(crate::spec::Take { n: 5, seed: 0 });
        assert!(check_size(&sampled, &reg, DEFAULT_MAX_GROUND).is_err());
    }
}
