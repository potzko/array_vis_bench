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
#[derive(Debug, Clone)]
enum Bound {
    Type(SpecNode),
    Const(String),
}

type Env = HashMap<String, Bound>;

/// The result of solving a query: the deduped ground trees plus any build-time
/// warnings (e.g. an `N` clamped to the available population). The caller
/// (build.rs / the generator) turns warnings into `cargo:warning=…` lines.
#[derive(Debug, Default)]
pub struct SolveOutput {
    pub sorts: Vec<SpecNode>,
    pub warnings: Vec<String>,
}

/// Solve a parsed query into its set of ground sort trees.
pub fn solve(query: &Query, reg: &Registry) -> Result<SolveOutput, String> {
    let mut warnings = Vec::new();

    // Process bindings in order, growing a set of environments (the
    // cross-product over every exhaustive/sampled hole). A shared variable is
    // just an earlier env entry every later binding reads — that is what makes
    // an arity-mismatched combination unrepresentable rather than merely
    // rejected later.
    let mut envs: Vec<Env> = vec![Env::new()];
    for binding in &query.bindings {
        let mut next = Vec::new();
        for env in &envs {
            for bound in eval_binding(binding, env, reg, query.depth, &mut warnings)? {
                if check_refinements(binding, &bound, env, reg)? {
                    let mut e = env.clone();
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

// ── binding / value evaluation ───────────────────────────────────────────────

fn eval_binding(
    binding: &Binding,
    env: &Env,
    reg: &Registry,
    depth: usize,
    warnings: &mut Vec<String>,
) -> Result<Vec<Bound>, String> {
    // A const variable: `let blk: Num = 32;` or `let blk2: Num = blk;`.
    match &binding.value {
        QValue::Const(lit) => return Ok(vec![Bound::Const(lit.clone())]),
        QValue::Ident(name) => {
            if let Some(Bound::Const(lit)) = env.get(name) {
                return Ok(vec![Bound::Const(lit.clone())]);
            }
        }
        _ => {}
    }
    // Otherwise it's a type binding; its role drives any hole.
    let nodes = eval_type(
        &binding.value,
        &binding.role,
        &binding.refinements,
        env,
        reg,
        depth,
        warnings,
    )?;
    Ok(nodes.into_iter().map(Bound::Type).collect())
}

/// Evaluate a value in TYPE context against `role`, returning every ground
/// subtree it stands for (one for a pinned value, many for a hole/family).
fn eval_type(
    value: &QValue,
    role: &str,
    refinements: &[Refinement],
    env: &Env,
    reg: &Registry,
    depth: usize,
    warnings: &mut Vec<String>,
) -> Result<Vec<SpecNode>, String> {
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
            apply_quant(pop, *quant, role, reg, warnings)
        }
        QValue::Ident(name) => match env.get(name) {
            Some(Bound::Type(node)) => Ok(vec![node.clone()]),
            Some(Bound::Const(_)) => {
                Err(format!("`{name}` is a const variable but a type is expected here"))
            }
            // Not a variable → a nullary component name.
            None => {
                reg.get(name)
                    .ok_or_else(|| format!("unknown component or variable `{name}`"))?;
                Ok(vec![SpecNode { name: name.clone(), args: vec![] }])
            }
        },
        QValue::App { name, args } => eval_app(name, args, env, reg, depth, warnings),
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
    reg: &Registry,
    depth: usize,
    warnings: &mut Vec<String>,
) -> Result<Vec<SpecNode>, String> {
    let comp = reg
        .get(name)
        .ok_or_else(|| format!("unknown component `{name}`"))?;

    // For each input argument, a list of the concrete `Arg`s it can become.
    let mut arg_options: Vec<Vec<Arg>> = Vec::new();
    for a in args {
        match a {
            QArg::Named { name: pname, value } => {
                let param = comp
                    .param(pname)
                    .ok_or_else(|| format!("`{name}` has no parameter `{pname}`"))?;
                match &param.kind {
                    ParamKind::Type { role, .. } => {
                        let opts = eval_type(value, role, &[], env, reg, depth, warnings)?;
                        arg_options.push(
                            opts.into_iter()
                                .map(|node| Arg::Slot { name: pname.clone(), value: node })
                                .collect(),
                        );
                    }
                    ParamKind::Const { default, values } => {
                        let opts = eval_const(value, Some((default, values)), env, warnings)?;
                        arg_options.push(
                            opts.into_iter()
                                .map(|v| Arg::NamedConst { name: pname.clone(), value: v })
                                .collect(),
                        );
                    }
                    ParamKind::Project { .. } => {
                        return Err(format!(
                            "`{pname}` of `{name}` is a structural (projected) param and \
                             cannot be passed as an argument — refine it on a binding instead"
                        ))
                    }
                }
            }
            QArg::Pos(value) => {
                let opts = eval_const(value, None, env, warnings)?;
                arg_options.push(opts.into_iter().map(Arg::Const).collect());
            }
        }
    }

    // Cartesian product of the per-argument option lists.
    let mut combos: Vec<Vec<Arg>> = vec![vec![]];
    for opts in &arg_options {
        let mut nextc = Vec::new();
        for combo in &combos {
            for o in opts {
                let mut c = combo.clone();
                c.push(o.clone());
                nextc.push(c);
            }
        }
        combos = nextc;
    }
    Ok(combos
        .into_iter()
        .map(|args| SpecNode { name: name.to_string(), args })
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
