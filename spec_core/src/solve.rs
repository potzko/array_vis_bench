//! Stage 0′ — the typed constraint SOLVER. Lowers a [`Query`] (let-bindings,
//! shared variables, refinements, and holes with quantifiers) into a *set* of
//! ground [`SpecNode`] trees. Pinned specs, partial families, and full
//! generation are then literally the same code path with different numbers of
//! holes — 0 holes → 1 sort, all holes → many, partial → a family.
//!
//! The decisive property: a *shared variable* threads ONE value to every use,
//! and a refinement (`Partition[pivot = p]`) adds a role constraint to that
//! value. So `quick_sort(partition = LL, pivot = ninther_dual)` with a shared
//! `p` is never *built* — LL's projected `pivot` is `PivotSingle`, a dual
//! selector doesn't provide it, and the combination is pruned at generation
//! time. rustc remains a redundant backstop, never the first line of defence.
//!
//! Discipline (enforced by construction): the only constraints are STRUCTURAL
//! EQUALITY (a shared variable: `a == b`) and ROLE MEMBERSHIP (`x: Role`). No
//! arithmetic, conditionals, or negation. Numbers are shared/defaulted/finitely
//! enumerated as opaque tokens — never reasoned about.

use std::collections::{HashMap, HashSet};

use crate::enumerate::enumerate;
use crate::registry::{ParamKind, Registry};
use crate::resolve::resolve;
use crate::spec::{Arg, Binding, QArg, QValue, Quant, Query, Refinement, SpecNode};

/// What a bound name carries: a type subtree (a sort/partition/pivot/…) or a
/// const literal (a shared number / bool).
#[derive(Debug, Clone, PartialEq)]
enum Bound {
    Type(SpecNode),
    Const(String),
}

type Env = HashMap<String, Bound>;

/// The captures (`as name`) a value introduced — a flat, program-global set of
/// variable bindings threaded OUT of evaluation. Merged into the environment so
/// later occurrences (sibling args, later bindings) see them; a rebind to a
/// different value prunes (`unify_caps` → `None`). This is the whole of the
/// capture mechanism — no scopes, no constraint store.
type Caps = HashMap<String, Bound>;

/// Unify two capture sets: equal where they overlap, else `None` (prune the
/// combination). This is what realizes "additive intersection" — two
/// occurrences of the same variable must agree.
fn unify_caps(base: &Caps, add: &Caps) -> Option<Caps> {
    let mut out = base.clone();
    for (k, v) in add {
        match out.get(k) {
            Some(existing) if existing != v => return None,
            Some(_) => {}
            None => {
                out.insert(k.clone(), v.clone());
            }
        }
    }
    Some(out)
}

/// The result of solving a query: the deduped ground trees plus any build-time
/// warnings (e.g. an `N` clamped to the available population). The caller
/// (build.rs / the generator) turns warnings into `cargo:warning=…` lines.
#[derive(Debug, Default)]
pub struct SolveOutput {
    pub sorts: Vec<SpecNode>,
    pub warnings: Vec<String>,
}

/// Solve a parsed query into its set of ground sort trees, with the default size
/// budget ([`crate::cardinality::DEFAULT_MAX_GROUND`]).
pub fn solve(query: &Query, reg: &Registry) -> Result<SolveOutput, String> {
    solve_within(query, reg, crate::cardinality::DEFAULT_MAX_GROUND)
}

/// Solve a query, refusing it up front if the static size analysis predicts a
/// materialized population larger than `budget`. This is the guard that stops an
/// unbounded recursive role or an unintended full cross-product from grinding
/// `enumerate` for minutes (or forever) — the cost is bounded BEFORE any tree is
/// built. Raise `budget` for a deliberately large sweep.
pub fn solve_within(query: &Query, reg: &Registry, budget: u128) -> Result<SolveOutput, String> {
    let mut warnings = Vec::new();

    // Size guard FIRST: estimate the query's peak materialization and bail with a
    // clear, actionable error before touching `enumerate`.
    let est = crate::cardinality::check_size(query, reg, budget)?;
    // Under budget but within a quarter of it: surface a heads-up rather than
    // letting a near-runaway pass silently (build scripts print warnings).
    if est.peak.exceeds(budget / 4) {
        warnings.push(format!(
            "large build: estimated peak materialization ~{} of a {budget} budget — \
             sample with `?N` or narrow the query if this is unintended",
            est.peak
        ));
    }

    // Static pass: collect capture-variable names and reject forward/cyclic
    // capture references. This lets the runtime treat any REMAINING unbound
    // capture-reference as "this branch never binds it" and prune, rather than
    // having to guess between "bound later" and "never bound".
    let cvars = check_capture_order(query)?;

    // Process bindings in order, growing a set of environments (the
    // cross-product over every exhaustive/sampled hole). A shared variable is
    // just an earlier env entry every later binding reads — that is what makes
    // an arity-mismatched combination unrepresentable rather than merely
    // rejected later.
    let mut envs: Vec<Env> = vec![Env::new()];
    for binding in &query.bindings {
        let mut next = Vec::new();
        for env in &envs {
            for (bound, caps) in eval_binding(binding, env, &cvars, reg, query.depth, &mut warnings)? {
                // Merge the captures this value introduced into the flat env,
                // unifying with anything already bound — a conflict prunes.
                let mut e = match unify_caps(env, &caps) {
                    Some(m) => m,
                    None => continue,
                };
                if check_refinements(binding, &bound, &e, reg)? {
                    e.insert(binding.name.clone(), bound);
                    next.push(e);
                }
            }
        }
        envs = next;
    }

    let target = &query.bindings.last().unwrap().name;
    let mut grounds = Vec::new();
    for env in envs {
        match env.get(target) {
            Some(Bound::Type(node)) => grounds.push(node.clone()),
            Some(Bound::Const(_)) => {
                return Err("the target binding must be a sort (a type), not a const".into())
            }
            None => unreachable!("target binding is the last one processed"),
        }
    }

    // Dedup on the RESOLVED canonical sort — two different spec trees that
    // elaborate to the same Rust type are one sort.
    grounds = dedup_canonical(grounds, reg)?;

    // `N of q`: N distinct WHOLE sorts, sampled from the deduped population.
    if let Some(take) = query.take {
        grounds = sample_distinct(grounds, take.n, take.seed, "query `N of`", reg, &mut warnings)?;
    }

    Ok(SolveOutput { sorts: grounds, warnings: dedup_preserving_order(warnings) })
}

// ── capture-variable static analysis (bind-before-use) ───────────────────────

/// Collect every capture-variable name (`… as name`) appearing anywhere in the
/// query.
fn collect_capture_vars(query: &Query) -> HashSet<String> {
    let mut out = HashSet::new();
    for b in &query.bindings {
        collect_caps_value(&b.value, &mut out);
    }
    out
}

fn collect_caps_value(v: &QValue, out: &mut HashSet<String>) {
    match v {
        QValue::Capture { name, inner } => {
            out.insert(name.clone());
            collect_caps_value(inner, out);
        }
        QValue::App { args, .. } => {
            for a in args {
                match a {
                    QArg::Named { value, .. } | QArg::Pos(value) => collect_caps_value(value, out),
                }
            }
        }
        QValue::Where { alts, .. } => alts.iter().for_each(|a| collect_caps_value(a, out)),
        QValue::Diff { base, subtrahends } => {
            collect_caps_value(base, out);
            subtrahends.iter().for_each(|s| collect_caps_value(s, out));
        }
        QValue::Set(members) => members.iter().for_each(|m| collect_caps_value(m, out)),
        QValue::Hole(_) | QValue::Ident(_) | QValue::Const(_) => {}
    }
}

/// Verify every reference to a capture-variable is preceded — in evaluation
/// order — by a capture of that name. Forward (use-before-capture) and cyclic
/// references are rejected here with a clear error. Returns the capture-var set,
/// which the solver uses to PRUNE (rather than error on) any reference left
/// unbound at runtime because the chosen union branch didn't capture it.
fn check_capture_order(query: &Query) -> Result<HashSet<String>, String> {
    let cvars = collect_capture_vars(query);
    let mut available: HashSet<String> = HashSet::new();
    for b in &query.bindings {
        for r in &b.refinements {
            check_order_value(&r.value, &cvars, &mut available)?;
        }
        check_order_value(&b.value, &cvars, &mut available)?;
        available.insert(b.name.clone());
    }
    Ok(cvars)
}

fn check_order_value(
    v: &QValue,
    cvars: &HashSet<String>,
    available: &mut HashSet<String>,
) -> Result<(), String> {
    match v {
        QValue::Ident(name) => {
            if cvars.contains(name) && !available.contains(name) {
                return Err(format!(
                    "`{name}` is referenced before it is captured — forward/cyclic capture \
                     references are not supported; capture it earlier in evaluation order"
                ));
            }
            Ok(())
        }
        QValue::Hole(_) | QValue::Const(_) => Ok(()),
        QValue::Capture { name, inner } => {
            check_order_value(inner, cvars, available)?; // inner first…
            available.insert(name.clone()); // …then `name` is bound
            Ok(())
        }
        QValue::App { args, .. } => {
            // Left-to-right: a sibling may reference a var an earlier sibling captured.
            for a in args {
                let val = match a {
                    QArg::Named { value, .. } | QArg::Pos(value) => value,
                };
                check_order_value(val, cvars, available)?;
            }
            Ok(())
        }
        QValue::Where { alts, .. } => {
            // Each branch is checked against the SAME incoming scope; a var
            // captured in ANY branch becomes available after the union.
            let mut after = available.clone();
            for alt in alts {
                let mut branch = available.clone();
                check_order_value(alt, cvars, &mut branch)?;
                for v in branch {
                    after.insert(v);
                }
            }
            *available = after;
            Ok(())
        }
        QValue::Diff { base, subtrahends } => {
            check_order_value(base, cvars, available)?;
            for s in subtrahends {
                check_order_value(s, cvars, available)?;
            }
            Ok(())
        }
        QValue::Set(members) => {
            for m in members {
                check_order_value(m, cvars, available)?;
            }
            Ok(())
        }
    }
}

// ── binding / value evaluation ───────────────────────────────────────────────

fn eval_binding(
    binding: &Binding,
    env: &Env,
    cvars: &HashSet<String>,
    reg: &Registry,
    depth: usize,
    warnings: &mut Vec<String>,
) -> Result<Vec<(Bound, Caps)>, String> {
    // A const variable: `let blk: Num = 32;` or `let blk2: Num = blk;`.
    match &binding.value {
        QValue::Const(lit) => return Ok(vec![(Bound::Const(lit.clone()), Caps::new())]),
        QValue::Ident(name) => {
            if let Some(Bound::Const(lit)) = env.get(name) {
                return Ok(vec![(Bound::Const(lit.clone()), Caps::new())]);
            }
        }
        _ => {}
    }
    // Otherwise it's a type binding; its role drives any hole.
    let cands = eval_type(
        &binding.value,
        &binding.role,
        &binding.refinements,
        env,
        cvars,
        reg,
        depth,
        warnings,
    )?;
    Ok(cands.into_iter().map(|(n, c)| (Bound::Type(n), c)).collect())
}

/// Evaluate a value in TYPE context against `role`, returning every ground
/// subtree it stands for (one for a pinned value, many for a hole/family).
fn eval_type(
    value: &QValue,
    role: &str,
    refinements: &[Refinement],
    env: &Env,
    cvars: &HashSet<String>,
    reg: &Registry,
    depth: usize,
    warnings: &mut Vec<String>,
) -> Result<Vec<(SpecNode, Caps)>, String> {
    match value {
        QValue::Hole(quant) => {
            // Population = every legal ground tree of this role (depth-bounded),
            // restricted to components that actually declare each refined param.
            // A unit partition that does not `project pivot` is excluded here —
            // that exclusion IS the arity filter.
            let mut pop: Vec<SpecNode> = enumerate(reg, role, depth)
                .into_iter()
                .filter(|node| local_refine_ok(node, refinements, reg))
                .collect();
            pop = dedup_canonical(pop, reg)?;
            // A bare hole introduces no captures.
            Ok(apply_quant(pop, *quant, role, reg, warnings)?
                .into_iter()
                .map(|n| (n, Caps::new()))
                .collect())
        }
        QValue::Ident(name) => match env.get(name) {
            Some(Bound::Type(node)) => Ok(vec![(node.clone(), Caps::new())]),
            Some(Bound::Const(_)) => {
                Err(format!("`{name}` is a const variable but a type is expected here"))
            }
            None => {
                // A capture variable unbound in THIS candidate's branch → prune.
                // The static `check_capture_order` pass guarantees it isn't a
                // forward reference, so "unbound here" means "this branch never
                // binds it" (e.g. the north-star's empty HeapExtraction branch).
                if cvars.contains(name) {
                    return Ok(vec![]);
                }
                // Otherwise a nullary component name.
                reg.get(name)
                    .ok_or_else(|| format!("unknown component or variable `{name}`"))?;
                Ok(vec![(SpecNode { name: name.clone(), args: vec![] }, Caps::new())])
            }
        },
        QValue::App { name, args } => eval_app(name, args, env, cvars, reg, depth, warnings),
        QValue::Capture { name, inner } => {
            // Evaluate the inner value, then bind each chosen node to `name`,
            // threading it out as a capture (unify if `name` already captured
            // within this subtree).
            let mut out = Vec::new();
            for (node, caps) in eval_type(inner, role, refinements, env, cvars, reg, depth, warnings)? {
                let mut one = Caps::new();
                one.insert(name.clone(), Bound::Type(node.clone()));
                if let Some(merged) = unify_caps(&caps, &one) {
                    out.push((node, merged));
                }
            }
            Ok(out)
        }
        QValue::Where { quant, alts } => {
            // A union: the population is exactly the alternatives' expansions
            // (each alt is a value in the same role/context), deduped on the
            // canonical type, then sampled by the base hole's quantifier. Each
            // candidate carries the captures of whichever alternative produced
            // it — so a capture inside one branch rides out with that branch.
            let mut cands: Vec<(SpecNode, Caps)> = Vec::new();
            for alt in alts {
                cands.extend(eval_type(alt, role, refinements, env, cvars, reg, depth, warnings)?);
            }
            let cands = dedup_canonical_caps(cands, reg)?;
            apply_quant_caps(cands, *quant, role, reg, warnings)
        }
        QValue::Set(members) => {
            // The union of the members' expansions (used as a subtrahend).
            let mut out = Vec::new();
            for m in members {
                out.extend(eval_type(m, role, &[], env, cvars, reg, depth, warnings)?);
            }
            dedup_canonical_caps(out, reg)
        }
        QValue::Diff { base, subtrahends } => {
            // base population MINUS every candidate whose canonical type appears
            // in a subtrahend's expansion. Membership only — no arithmetic.
            // The subtraction is fused into the population build: removed members
            // are skipped AS the base is enumerated, so it happens BEFORE any
            // sampling quantifier and each candidate is resolved once.
            let mut remove: HashSet<String> = HashSet::new();
            for sub in subtrahends {
                for (node, _caps) in eval_type(sub, role, &[], env, cvars, reg, depth, warnings)? {
                    remove.insert(resolve(&node, reg)?.type_expr);
                }
            }
            match base.as_ref() {
                // A hole base: enumerate the role exhaustively, drop removed
                // members during the build, dedup, THEN apply the hole's
                // quantifier (subtract-then-sample, never sample-then-subtract).
                QValue::Hole(quant) => {
                    let mut keep: Vec<(SpecNode, Caps)> = Vec::new();
                    let mut seen: HashSet<String> = HashSet::new();
                    for node in enumerate(reg, role, depth) {
                        if !local_refine_ok(&node, refinements, reg) {
                            continue;
                        }
                        let ty = resolve(&node, reg)?.type_expr;
                        if remove.contains(&ty) {
                            continue;
                        }
                        if seen.insert(ty) {
                            keep.push((node, Caps::new()));
                        }
                    }
                    apply_quant_caps(keep, *quant, role, reg, warnings)
                }
                // A non-hole base (a specific value, a nested diff, …): evaluate
                // it, then filter. No extra quantifier.
                _ => {
                    let base_cands =
                        eval_type(base, role, refinements, env, cvars, reg, depth, warnings)?;
                    let mut out = Vec::new();
                    for (node, caps) in base_cands {
                        if !remove.contains(&resolve(&node, reg)?.type_expr) {
                            out.push((node, caps));
                        }
                    }
                    Ok(out)
                }
            }
        }
        QValue::Const(c) => {
            Err(format!("const literal `{c}` cannot be used where a type ({role}) is expected"))
        }
    }
}

/// Build the ground trees of a component application, cross-producting over any
/// holes inside its arguments.
fn eval_app(
    name: &str,
    args: &[QArg],
    env: &Env,
    cvars: &HashSet<String>,
    reg: &Registry,
    depth: usize,
    warnings: &mut Vec<String>,
) -> Result<Vec<(SpecNode, Caps)>, String> {
    let comp = reg
        .get(name)
        .ok_or_else(|| format!("unknown component `{name}`"))?;

    // Left-to-right fold over the arguments, carrying each partial combo's
    // accumulated captures. Evaluating a later arg against `env + caps-so-far`
    // is what lets a sibling argument REFERENCE a variable an earlier sibling
    // CAPTURED (`combined<a = _ as x, b = x>`); `unify_caps` prunes any combo
    // where two occurrences disagree. With no captures this is exactly the old
    // Cartesian product.
    let mut combos: Vec<(Vec<Arg>, Caps)> = vec![(Vec::new(), Caps::new())];
    for a in args {
        let mut nextc = Vec::new();
        for (args_so_far, caps_so_far) in &combos {
            // Effective environment for this arg = base env + captures so far.
            let mut eenv = env.clone();
            for (k, v) in caps_so_far {
                eenv.insert(k.clone(), v.clone());
            }
            let pieces: Vec<(Arg, Caps)> = match a {
                QArg::Named { name: pname, value } => {
                    let param = comp
                        .param(pname)
                        .ok_or_else(|| format!("`{name}` has no parameter `{pname}`"))?;
                    match &param.kind {
                        ParamKind::Type { role, .. } => {
                            eval_type(value, role, &[], &eenv, cvars, reg, depth, warnings)?
                                .into_iter()
                                .map(|(node, caps)| {
                                    (Arg::Slot { name: pname.clone(), value: node }, caps)
                                })
                                .collect()
                        }
                        ParamKind::Const { default, values } => {
                            eval_const(value, Some((default, values)), &eenv, warnings)?
                                .into_iter()
                                .map(|v| {
                                    (Arg::NamedConst { name: pname.clone(), value: v }, Caps::new())
                                })
                                .collect()
                        }
                        ParamKind::Project { .. } => {
                            return Err(format!(
                                "`{pname}` of `{name}` is a structural (projected) param and \
                                 cannot be passed as an argument — refine it on a binding instead"
                            ))
                        }
                    }
                }
                QArg::Pos(value) => eval_const(value, None, &eenv, warnings)?
                    .into_iter()
                    .map(|v| (Arg::Const(v), Caps::new()))
                    .collect(),
            };
            for (arg_piece, arg_caps) in pieces {
                if let Some(merged) = unify_caps(caps_so_far, &arg_caps) {
                    let mut c = args_so_far.clone();
                    c.push(arg_piece);
                    nextc.push((c, merged));
                }
            }
        }
        combos = nextc;
    }
    Ok(combos
        .into_iter()
        .map(|(args, caps)| (SpecNode { name: name.to_string(), args }, caps))
        .collect())
}

/// Evaluate a value in CONST context, returning every literal it stands for.
/// `param` (when known) supplies the declared value-set + default a const hole
/// ranges over.
fn eval_const(
    value: &QValue,
    param: Option<(&Option<String>, &Vec<String>)>,
    env: &Env,
    warnings: &mut Vec<String>,
) -> Result<Vec<String>, String> {
    match value {
        QValue::Const(lit) => Ok(vec![lit.clone()]),
        QValue::Ident(name) => match env.get(name) {
            Some(Bound::Const(lit)) => Ok(vec![lit.clone()]),
            Some(Bound::Type(_)) => {
                Err(format!("`{name}` is a type variable but a const is expected here"))
            }
            None => Err(format!("unknown const variable `{name}`")),
        },
        QValue::Hole(quant) => {
            let (default, values) = param.ok_or(
                "a const hole must be a NAMED argument so its value set is known \
                 (positional const holes are unsupported)",
            )?;
            // Population is the declared "neat values"; with none declared, a
            // hole degenerates to the single default. Pure membership — the
            // solver never reasons about the numbers themselves.
            let pop: Vec<String> = if values.is_empty() {
                default.clone().into_iter().collect()
            } else {
                values.clone()
            };
            apply_quant_str(pop, *quant, warnings)
        }
        QValue::App { name, .. } => {
            Err(format!("`{name}(…)` is a component but a const literal is expected here"))
        }
        QValue::Capture { .. } => {
            Err("capturing const values is not supported yet".into())
        }
        QValue::Where { .. } => {
            Err("`where ( … )` unions are only valid in type position, not for a const".into())
        }
        QValue::Diff { .. } | QValue::Set { .. } => {
            Err("set difference / set literals are only valid in type position, not for a const".into())
        }
    }
}

// ── refinement constraint solving ────────────────────────────────────────────

/// Is `node`'s component eligible to be refined on every named param? (Used to
/// pre-filter a hole's population; the authoritative role check is
/// `check_refinements`.)
fn local_refine_ok(node: &SpecNode, refinements: &[Refinement], reg: &Registry) -> bool {
    match reg.get(&node.name) {
        Some(c) => refinements.iter().all(|r| c.param(&r.param).is_some()),
        None => false,
    }
}

/// The cross-binding constraint check: for each refinement on this binding, the
/// chosen filler must declare the param, and the value threaded in (a shared
/// variable) must provide that param's role. Returning `Ok(false)` PRUNES the
/// combination — it is never produced.
fn check_refinements(
    binding: &Binding,
    bound: &Bound,
    env: &Env,
    reg: &Registry,
) -> Result<bool, String> {
    let node = match bound {
        Bound::Type(node) => node,
        Bound::Const(_) => {
            return if binding.refinements.is_empty() {
                Ok(true)
            } else {
                Err(format!("const binding `{}` cannot carry refinements", binding.name))
            }
        }
    };
    let comp = reg
        .get(&node.name)
        .ok_or_else(|| format!("unknown component `{}`", node.name))?;

    for r in &binding.refinements {
        // The filler must declare the projected/typed param at all…
        let role = match comp.param_role(&r.param) {
            Some(role) => role.to_string(),
            None => {
                if comp.param(&r.param).is_none() {
                    return Ok(false); // e.g. a unit partition has no `.pivot`
                }
                return Err(format!(
                    "refinement `{}` of `{}` targets a const param, which is not a role \
                     constraint",
                    r.param, node.name
                ));
            }
        };
        // …and whatever is threaded in must provide that role.
        let val_node = resolve_ref_value(&r.value, env, reg)?;
        let val_comp = reg
            .get(&val_node.name)
            .ok_or_else(|| format!("unknown component `{}`", val_node.name))?;
        if !val_comp.provides.contains(&role) {
            return Ok(false); // arity mismatch → UNREPRESENTABLE
        }
    }
    Ok(true)
}

/// A refinement's right-hand side must be a single concrete node: a shared
/// variable or a nullary component.
fn resolve_ref_value(value: &QValue, env: &Env, reg: &Registry) -> Result<SpecNode, String> {
    match value {
        QValue::Ident(name) => match env.get(name) {
            Some(Bound::Type(node)) => Ok(node.clone()),
            Some(Bound::Const(_)) => {
                Err(format!("refinement value `{name}` is a const, expected a type"))
            }
            None => {
                reg.get(name)
                    .ok_or_else(|| format!("unknown component or variable `{name}`"))?;
                Ok(SpecNode { name: name.clone(), args: vec![] })
            }
        },
        _ => Err("a refinement value must be a variable or a component name".into()),
    }
}

// ── quantifier application (seeded sampling + clamp/warn) ─────────────────────

fn apply_quant(
    pop: Vec<SpecNode>,
    quant: Quant,
    role: &str,
    reg: &Registry,
    warnings: &mut Vec<String>,
) -> Result<Vec<SpecNode>, String> {
    match quant {
        Quant::Exhaustive => Ok(pop),
        Quant::One { seed } => sample_distinct(pop, 1, seed, &format!("hole `{role}`"), reg, warnings),
        Quant::N { n, seed } => {
            sample_distinct(pop, n, seed, &format!("hole `{role}`"), reg, warnings)
        }
    }
}

fn apply_quant_str(
    pop: Vec<String>,
    quant: Quant,
    warnings: &mut Vec<String>,
) -> Result<Vec<String>, String> {
    let (n, seed) = match quant {
        Quant::Exhaustive => return Ok(dedup_preserving_order(pop)),
        Quant::One { seed } => (1, seed),
        Quant::N { n, seed } => (n, seed),
    };
    let pop = dedup_preserving_order(pop);
    Ok(sample_indices(pop, n, seed, "const hole", warnings))
}

/// Sample `n` DISTINCT trees (deduped on canonical form) with a seeded shuffle.
/// `n` larger than the population is clamped, with a warning — never silent.
fn sample_distinct(
    pop: Vec<SpecNode>,
    n: usize,
    seed: u64,
    label: &str,
    reg: &Registry,
    warnings: &mut Vec<String>,
) -> Result<Vec<SpecNode>, String> {
    let pop = dedup_canonical(pop, reg)?;
    Ok(sample_indices(pop, n, seed, label, warnings))
}

/// Seeded distinct sampling over an already-deduped population.
fn sample_indices<T: Clone>(
    pop: Vec<T>,
    n: usize,
    seed: u64,
    label: &str,
    warnings: &mut Vec<String>,
) -> Vec<T> {
    if n >= pop.len() {
        if n > pop.len() {
            warnings.push(format!(
                "{label}: requested {n} but only {} available — clamped to {}",
                pop.len(),
                pop.len()
            ));
        }
        return pop;
    }
    let mut rng = Rng::new(seed);
    let len = pop.len();
    let mut idx: Vec<usize> = (0..len).collect();
    // Partial Fisher–Yates: the first `n` slots become a uniform sample.
    for i in 0..n {
        let j = i + rng.below(len - i);
        idx.swap(i, j);
    }
    idx[..n].iter().map(|&i| pop[i].clone()).collect()
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Dedup spec trees by the Rust type they elaborate to (the canonical form).
fn dedup_canonical(nodes: Vec<SpecNode>, reg: &Registry) -> Result<Vec<SpecNode>, String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    for node in nodes {
        let r = resolve(&node, reg)?;
        if seen.insert(r.type_expr) {
            out.push(node);
        }
    }
    Ok(out)
}

/// Like [`dedup_canonical`] but for capture-carrying candidates (a union's
/// members). First occurrence of a canonical type wins — including its captures.
fn dedup_canonical_caps(
    cands: Vec<(SpecNode, Caps)>,
    reg: &Registry,
) -> Result<Vec<(SpecNode, Caps)>, String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    for (node, caps) in cands {
        let r = resolve(&node, reg)?;
        if seen.insert(r.type_expr) {
            out.push((node, caps));
        }
    }
    Ok(out)
}

/// Apply a quantifier to an already-deduped, capture-carrying population.
fn apply_quant_caps(
    pop: Vec<(SpecNode, Caps)>,
    quant: Quant,
    role: &str,
    _reg: &Registry,
    warnings: &mut Vec<String>,
) -> Result<Vec<(SpecNode, Caps)>, String> {
    match quant {
        Quant::Exhaustive => Ok(pop),
        Quant::One { seed } => Ok(sample_indices(pop, 1, seed, &format!("union `{role}`"), warnings)),
        Quant::N { n, seed } => {
            Ok(sample_indices(pop, n, seed, &format!("union `{role}`"), warnings))
        }
    }
}

fn dedup_preserving_order<T: Clone + Eq + std::hash::Hash>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for it in items {
        if seen.insert(it.clone()) {
            out.push(it);
        }
    }
    out
}

/// SplitMix64 — a tiny seeded PRNG so every random quantifier is reproducible
/// across builds (no `rand`, no clock, pure std).
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}
