//! The AVBS **query** language — pest frontend.
//!
//! This is the formal grammar for the surface the user authors (see
//! [`avbs.pest`](./avbs.pest)). It is *parse-only* for now: it proves the
//! grammar accepts the target shape (the north-star) before any AST or solver
//! wiring. The existing hand-written `spec::parse_query` still drives the live
//! pipeline; this module exists to lock the language shape.
//!
//! Gated behind the `pest-frontend` feature so the default engine build (the
//! proc-macro and the codegen build scripts) stays dependency-free.

use crate::registry::Registry;
use crate::spec::{Binding, QArg, QValue, Quant, Query, DEFAULT_DEPTH};
use pest::iterators::{Pair, Pairs};
use std::collections::HashMap;

#[derive(pest_derive::Parser)]
#[grammar = "avbs.pest"]
pub struct AvbsParser;

/// Parse a whole AVBS query program (one or more `Name: Type = Value` bindings,
/// preceded by optional `depth`/`N of` directives). Returns the raw pest pairs;
/// AST lowering is a later phase.
pub fn parse(text: &str) -> Result<Pairs<'_, Rule>, Box<pest::error::Error<Rule>>> {
    use pest::Parser;
    AvbsParser::parse(Rule::query, text).map_err(Box::new)
}

// ─────────────────────────────────────────────────────────────────────────────
//  Lowering — STEP 1 (the shell slice).
//
//  "Reuse + extend": lower the parsed AVBS program onto the EXISTING
//  `Query`/`QValue` AST so the existing `solve` runs unchanged. Only the subset
//  the current solver already supports is lowered here; new constructs (`as`
//  capture, `where` unions, `_ - {X}` set-difference, lists) error cleanly until
//  their solver phases land. Each emittable binding (one wrapped in a role
//  builtin like `SortingAlgorithm<…>`) becomes its OWN `Query`; cross-binding
//  shared variables and `List<…>` aggregation are a later step.
// ─────────────────────────────────────────────────────────────────────────────

/// Role builtins that wrap a concrete type: `SortingAlgorithm<X>` marks `X` as an
/// emittable sort (driver category `Sort`); `Selection<X>` marks `X` as an
/// emittable quick-select (driver category `QuickSelect`) — the first non-sort
/// first-class kind. `List`/`Mains` are reserved for the aggregation/consumer
/// track and not yet lowerable on the algorithm track.
fn is_wrapper_name(name: &str) -> bool {
    matches!(name, "SortingAlgorithm" | "Selection" | "List" | "Mains")
}

/// The role a wrapper contributes to the binding it heads. A wrapper exists per
/// first-class algorithm kind; the role string matches what the kind's driver
/// component `provides` (and the emit backend keys its per-kind body on the
/// driver's `category`).
fn wrapper_role(name: &str) -> Result<String, String> {
    match name {
        "SortingAlgorithm" => Ok("Sort".to_string()),
        "Selection" => Ok("QuickSelect".to_string()),
        other => Err(format!("`{other}<…>` is not lowerable yet (lists/consumers)")),
    }
}

/// The leading identifier of a `type` template, e.g. `ShellSort<{seq}>` → `ShellSort`.
fn type_head(type_tmpl: &str) -> &str {
    let t = type_tmpl.trim();
    let end = t.find(|c: char| c == '<' || c.is_whitespace()).unwrap_or(t.len());
    &t[..end]
}

/// Index: Rust type-head → component name. (On head collisions — e.g. baked
/// `QuickSort<…>` entries — first-wins; that ambiguity is the deferred quicksort
/// decomposition fork and doesn't arise for the shell slice.)
fn type_head_index(reg: &Registry) -> HashMap<String, String> {
    let mut m = HashMap::new();
    for c in reg.iter() {
        m.entry(type_head(&c.type_tmpl).to_string())
            .or_insert_with(|| c.name.clone());
    }
    m
}

/// Lower a whole AVBS program to one `Query` per emittable binding.
pub fn lower_program(text: &str, reg: &Registry) -> Result<Vec<Query>, String> {
    let mut pairs = parse(text).map_err(|e| e.to_string())?;
    let query = pairs.next().ok_or("empty parse")?; // Rule::query
    let idx = type_head_index(reg);

    let mut depth = DEFAULT_DEPTH;
    let mut take = None;
    let mut queries = Vec::new();
    for p in query.into_inner() {
        match p.as_rule() {
            Rule::depth_dir => depth = int_pair(first_inner(p)?)?,
            Rule::take_dir => take = Some(lower_take(p)?),
            Rule::binding => {
                let (binding, _emittable) = lower_binding(p, reg, &idx)?;
                queries.push(Query { depth, take, bindings: vec![binding] });
            }
            Rule::EOI => {}
            other => return Err(format!("unexpected {other:?} at program top level")),
        }
    }
    if queries.is_empty() {
        return Err("program has no emittable bindings".into());
    }
    Ok(queries)
}

// ─────────────────────────────────────────────────────────────────────────────
//  Whole-program lowering — the consumers / list / Mains track.
//
//  `lower_program` (above) returns one `Query` per binding for the algorithm
//  slices. `lower` returns the richer [`Program`]: algorithm families, named sets
//  (`List<SortingAlgorithm> = [A, B]`), and consumer declarations
//  (`consumers: List<Mains<Sorts>> = [visualiser, …]`). This is what connects the
//  language's `consumers:` binding to the runtime `Main` impls (the last loop:
//  query → solve → emit `AlgorithmEntry` → `Main` consumers run over them).
// ─────────────────────────────────────────────────────────────────────────────

/// A whole lowered AVBS program: the algorithm families, the named aggregations
/// over them, and the consumer pipelines that run across them.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// `Name: SortingAlgorithm = SortingAlgorithm<…>` — each an emittable family.
    pub algos: Vec<NamedAlgo>,
    /// `Name: List<SortingAlgorithm> = [A, B]` — a union over algo / set names.
    pub sets: Vec<NamedSet>,
    /// `name: List<Mains<Target>> = [visualiser, …]` — consumers over `Target`.
    pub consumers: Vec<ConsumerDecl>,
}

/// An emittable algorithm family binding and the `Query` it lowers to.
#[derive(Debug, Clone, PartialEq)]
pub struct NamedAlgo {
    pub name: String,
    pub query: Query,
}

/// A named aggregation: `Name: List<…> = [members]`, where each member names an
/// earlier algorithm family or another set.
#[derive(Debug, Clone, PartialEq)]
pub struct NamedSet {
    pub name: String,
    pub members: Vec<String>,
}

/// A consumer pipeline declaration: `name: List<Mains<Target>> = [mains…]`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConsumerDecl {
    /// The binding name (e.g. `consumers`).
    pub name: String,
    /// The set / algo this consumer list runs over (the `Mains<Target>` inner).
    pub target: String,
    /// The consumer names, in order (e.g. `[visualiser, correctness, benchmark]`).
    pub mains: Vec<String>,
}

/// A consumer resolved against a registry: its `Main` names paired with the
/// concrete algorithm LABELS it runs over (the same labels the emitted
/// `AlgorithmEntry` rows carry). The runtime maps `mains` → `Main` impls and
/// `algo_labels` → registered `&AlgorithmEntry` rows, then runs them.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedConsumer {
    pub name: String,
    pub mains: Vec<String>,
    pub algo_labels: Vec<String>,
}

impl Program {
    pub fn algo(&self, name: &str) -> Option<&NamedAlgo> {
        self.algos.iter().find(|a| a.name == name)
    }
    pub fn set(&self, name: &str) -> Option<&NamedSet> {
        self.sets.iter().find(|s| s.name == name)
    }

    /// Flatten a name (an algo binding or a set) to the algo-binding names it
    /// transitively covers, in order. Diamonds / cycles are visited once.
    pub fn resolve_algos(&self, name: &str) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();
        self.collect_algos(name, &mut out, &mut seen)?;
        Ok(out)
    }

    fn collect_algos(
        &self,
        name: &str,
        out: &mut Vec<String>,
        seen: &mut std::collections::HashSet<String>,
    ) -> Result<(), String> {
        if !seen.insert(name.to_string()) {
            return Ok(()); // already visited (diamond / cycle) — skip
        }
        if self.algo(name).is_some() {
            out.push(name.to_string());
            return Ok(());
        }
        if let Some(s) = self.set(name) {
            for m in &s.members {
                self.collect_algos(m, out, seen)?;
            }
            return Ok(());
        }
        Err(format!("`{name}` is not an algorithm or list binding in this program"))
    }

    /// Resolve every consumer against `reg`: solve each referenced algorithm
    /// family and collect the concrete labels, so the runtime can match them to
    /// registered `AlgorithmEntry` rows. Solving runs through the size guard, so a
    /// runaway family is rejected here too.
    pub fn resolve_consumers(&self, reg: &Registry) -> Result<Vec<ResolvedConsumer>, String> {
        let mut out = Vec::new();
        for c in &self.consumers {
            let algo_names = self.resolve_algos(&c.target)?;
            let mut labels: Vec<String> = Vec::new();
            for an in &algo_names {
                let algo = self.algo(an).expect("resolve_algos only yields algo names");
                let solved = crate::solve::solve(&algo.query, reg)?;
                for node in &solved.sorts {
                    labels.push(crate::resolve::resolve(node, reg)?.label);
                }
            }
            let mut seen = std::collections::HashSet::new();
            labels.retain(|l| seen.insert(l.clone())); // dedup, preserve order
            out.push(ResolvedConsumer {
                name: c.name.clone(),
                mains: c.mains.clone(),
                algo_labels: labels,
            });
        }
        Ok(out)
    }
}

/// Lower a whole AVBS program into a [`Program`]: algorithm families, named sets,
/// and consumer declarations. Unlike [`lower_program`] (one `Query` per binding,
/// rejecting lists/consumers), this lowers the full surface.
pub fn lower(text: &str, reg: &Registry) -> Result<Program, String> {
    let mut pairs = parse(text).map_err(|e| e.to_string())?;
    let query = pairs.next().ok_or("empty parse")?;
    let idx = type_head_index(reg);

    let mut depth = DEFAULT_DEPTH;
    let mut take = None;
    let mut prog = Program { algos: Vec::new(), sets: Vec::new(), consumers: Vec::new() };
    // Helper bindings (`p: Pivot = _`, `part: Partition[pivot = p] = _`) accumulate
    // and are prepended to every later emittable family's `Query`, so a family can
    // share variables across its slots (arity coupling). Names are program-global,
    // so `declared` rejects any duplicate (helper or otherwise) up front.
    let mut helpers: Vec<Binding> = Vec::new();
    let mut declared: std::collections::HashSet<String> = std::collections::HashSet::new();

    for p in query.into_inner() {
        match p.as_rule() {
            Rule::depth_dir => depth = int_pair(first_inner(p)?)?,
            Rule::take_dir => take = Some(lower_take(p)?),
            Rule::binding => {
                classify_binding(p, depth, take, reg, &idx, &mut prog, &mut helpers, &mut declared)?
            }
            Rule::EOI => {}
            other => return Err(format!("unexpected {other:?} at program top level")),
        }
    }
    if prog.algos.is_empty() && prog.sets.is_empty() && prog.consumers.is_empty() {
        return Err("program has no emittable / set / consumer bindings".into());
    }
    Ok(prog)
}

/// Classify a binding by its type annotation and value: `List<Mains<…>>` → a
/// consumer list, `List<…>` → a named set, a `SortingAlgorithm<…>` value → an
/// emittable family (with the accumulated helpers prepended), anything else → a
/// helper binding that feeds later families.
#[allow(clippy::too_many_arguments)]
fn classify_binding(
    p: Pair<Rule>,
    depth: usize,
    take: Option<crate::spec::Take>,
    reg: &Registry,
    idx: &HashMap<String, String>,
    prog: &mut Program,
    helpers: &mut Vec<Binding>,
    declared: &mut std::collections::HashSet<String>,
) -> Result<(), String> {
    let bp = p.clone();
    let mut it = p.into_inner();
    let name = it.next().ok_or("binding needs a name")?.as_str().to_string();
    let type_ann = it.next().ok_or("binding needs a type annotation")?;
    // The value follows the optional refinements.
    let next = it.next().ok_or("binding needs a value")?;
    let value = if next.as_rule() == Rule::refinements {
        it.next().ok_or("binding needs a value")?
    } else {
        next
    };

    if !declared.insert(name.clone()) {
        return Err(format!("duplicate binding name `{name}` — names must be unique"));
    }

    let atoms = parse_type_ann(type_ann)?;
    let primary = atoms.first().ok_or("empty type annotation")?;

    if primary.head == "List" {
        let elem = primary.args.first().ok_or("`List<…>` needs an element type")?;
        if elem.head == "Mains" {
            let target =
                elem.args.first().ok_or("`Mains<…>` needs a target set")?.head.clone();
            let mains = value_as_list_idents(value)?;
            prog.consumers.push(ConsumerDecl { name, target, mains });
        } else {
            let members = value_as_list_idents(value)?;
            prog.sets.push(NamedSet { name, members });
        }
        return Ok(());
    }

    let (binding, emittable) = lower_binding(bp, reg, idx)?;
    if emittable {
        // Prepend the helpers declared so far so the family's slots can share
        // their variables (the proven shared-pivot / refinement coupling).
        let mut bindings = helpers.clone();
        bindings.push(binding);
        prog.algos.push(NamedAlgo { name, query: Query { depth, take, bindings } });
    } else {
        helpers.push(binding);
    }
    Ok(())
}

/// A parsed type annotation node (a name-based mirror of a Rust generic type).
struct TyAnn {
    head: String,
    args: Vec<TyAnn>,
}

/// Parse a `type_ann` pair into its intersection atoms (`A & B` → `[A, B]`).
fn parse_type_ann(p: Pair<Rule>) -> Result<Vec<TyAnn>, String> {
    let mut atoms = Vec::new();
    for atom in p.into_inner() {
        atoms.push(parse_type_atom(atom)?);
    }
    Ok(atoms)
}

fn parse_type_atom(p: Pair<Rule>) -> Result<TyAnn, String> {
    match p.as_rule() {
        Rule::underscore => Ok(TyAnn { head: "_".into(), args: vec![] }),
        Rule::type_app => {
            let mut it = p.into_inner();
            let head = it.next().ok_or("type needs a head")?.as_str().to_string();
            let mut args = Vec::new();
            for inner in it {
                // each generic argument is itself a `type_ann`; take its first atom.
                let sub = parse_type_ann(inner)?;
                args.push(sub.into_iter().next().ok_or("empty generic argument")?);
            }
            Ok(TyAnn { head, args })
        }
        other => Err(format!("unexpected type atom {other:?}")),
    }
}

/// Extract `[a, b, c]` as a list of bare identifiers (no captures, no `-`).
fn value_as_list_idents(value: Pair<Rule>) -> Result<Vec<String>, String> {
    let mut it = value.into_inner();
    let unary = it.next().ok_or("empty value")?;
    if it.next().is_some() {
        return Err("a list binding's value cannot carry a capture or `where`".into());
    }
    let mut uit = unary.into_inner();
    let first = uit.next().ok_or("empty unary")?;
    if uit.next().is_some() {
        return Err("a list binding's value cannot be a set difference".into());
    }
    if first.as_rule() != Rule::list_lit {
        return Err("expected a `[…]` list value".into());
    }
    let mut out = Vec::new();
    for member in first.into_inner() {
        out.push(value_as_single_ident(member)?);
    }
    Ok(out)
}

fn value_as_single_ident(value: Pair<Rule>) -> Result<String, String> {
    let unary = value.into_inner().next().ok_or("empty list member")?;
    let term = unary.into_inner().next().ok_or("empty list member")?;
    match term.as_rule() {
        Rule::application => {
            let mut it = term.into_inner();
            let head = it.next().ok_or("list member needs a name")?.as_str().to_string();
            if it.next().is_some() {
                return Err(format!("list member `{head}` must be a bare name"));
            }
            Ok(head)
        }
        other => Err(format!("a list member must be a bare name, found {other:?}")),
    }
}

fn lower_take(p: Pair<Rule>) -> Result<crate::spec::Take, String> {
    // take_dir = integer ~ "of" ~ seed?
    let mut it = p.into_inner();
    let n = int_pair(it.next().ok_or("take needs a count")?)?;
    let seed = match it.next() {
        Some(s) => seed_value(s)?,
        None => 0,
    };
    Ok(crate::spec::Take { n, seed })
}

/// Lower a binding to a [`Binding`] plus whether it is EMITTABLE (its value is a
/// `SortingAlgorithm<…>` wrapper → it becomes its own algorithm family). A
/// non-wrapper binding (`p: Pivot = _`) is a helper that feeds later families.
fn lower_binding(
    p: Pair<Rule>,
    reg: &Registry,
    idx: &HashMap<String, String>,
) -> Result<(Binding, bool), String> {
    // binding = ident ~ ":" ~ type_ann ~ refinements? ~ "=" ~ value
    let mut it = p.into_inner();
    let name = it.next().ok_or("binding needs a name")?.as_str().to_string();
    let type_ann = it.next().ok_or("binding needs a type annotation")?;
    // The next pair is either the optional `refinements` or the value.
    let next = it.next().ok_or("binding needs a value")?;
    let (refinements, value_pair) = if next.as_rule() == Rule::refinements {
        (lower_refinements(next, reg, idx)?, it.next().ok_or("binding needs a value")?)
    } else {
        (Vec::new(), next)
    };

    let raw = lower_value(value_pair, reg, idx)?;
    // Unwrap a top-level role wrapper (`SortingAlgorithm<X>`) → role + inner value
    // (and mark the binding emittable); otherwise it's a helper binding.
    let (role, value, emittable) = match raw {
        QValue::App { name: w, args } if is_wrapper_name(&w) => {
            let role = wrapper_role(&w)?;
            let inner = match args.into_iter().next() {
                Some(QArg::Pos(v)) | Some(QArg::Named { value: v, .. }) => v,
                None => return Err(format!("`{w}<…>` takes one type argument")),
            };
            (role, inner, true)
        }
        other => (role_from_type_ann(&type_ann)?, other, false),
    };
    Ok((Binding { name, role, refinements, value }, emittable))
}

/// Lower `[param = value, …]` into the solver's [`Refinement`]s. Each value is a
/// shared-variable reference (or nullary component) the solver role-checks
/// against the chosen filler's projected param.
fn lower_refinements(
    p: Pair<Rule>,
    reg: &Registry,
    idx: &HashMap<String, String>,
) -> Result<Vec<crate::spec::Refinement>, String> {
    let mut out = Vec::new();
    for r in p.into_inner() {
        // refinement = ident ~ "=" ~ value
        let mut it = r.into_inner();
        let param = it.next().ok_or("refinement needs a param name")?.as_str().to_string();
        let value = lower_value(it.next().ok_or("refinement needs a value")?, reg, idx)?;
        out.push(crate::spec::Refinement { param, value });
    }
    Ok(out)
}

/// The binding role from its annotation (used only when the value isn't a wrapper).
fn role_from_type_ann(p: &Pair<Rule>) -> Result<String, String> {
    let first = p.clone().into_inner().next().ok_or("empty type annotation")?;
    match first.as_rule() {
        Rule::underscore => {
            Err("cannot infer the binding role from `_` (value is not a role wrapper)".into())
        }
        Rule::type_app => {
            let head = first.into_inner().next().unwrap().as_str().to_string();
            if is_wrapper_name(&head) {
                wrapper_role(&head)
            } else {
                Ok(head) // a bare role name, e.g. `p: Pivot = …`
            }
        }
        other => Err(format!("unexpected type annotation {other:?}")),
    }
}

fn lower_value(p: Pair<Rule>, reg: &Registry, idx: &HashMap<String, String>) -> Result<QValue, String> {
    // value = unary ~ capture? ~ where_clause?
    // Collect the (optional) capture name and where-alternatives first, then
    // apply them in semantic order: the `where` rewrites the hole's population,
    // and the capture binds the final chosen value (so capture is OUTERMOST).
    let mut inner = p.into_inner();
    let base = lower_unary(inner.next().ok_or("empty value")?, reg, idx)?;
    let mut capture_name: Option<String> = None;
    let mut where_alts: Option<Vec<QValue>> = None;
    for extra in inner {
        match extra.as_rule() {
            Rule::capture => {
                capture_name =
                    Some(extra.into_inner().next().ok_or("capture needs a name")?.as_str().to_string());
            }
            Rule::where_clause => {
                // where_clause = "where" ~ "(" ~ value ~ ("|" ~ value)* ~ ")"
                let mut alts = Vec::new();
                for v in extra.into_inner() {
                    alts.push(lower_value(v, reg, idx)?);
                }
                where_alts = Some(alts);
            }
            other => return Err(format!("unexpected value suffix {other:?}")),
        }
    }

    let mut val = base;
    if let Some(alts) = where_alts {
        // The union takes its quantifier from the hole base it replaces.
        let quant = match val {
            QValue::Hole(q) => q,
            _ => {
                return Err(
                    "`where ( … )` requires a hole base, e.g. `_ where ( … )` or `?3 where ( … )`"
                        .into(),
                )
            }
        };
        val = QValue::Where { quant, alts };
    }
    if let Some(name) = capture_name {
        val = QValue::Capture { name, inner: Box::new(val) };
    }
    Ok(val)
}

fn lower_unary(p: Pair<Rule>, reg: &Registry, idx: &HashMap<String, String>) -> Result<QValue, String> {
    // unary = term ~ ("-" ~ term)*  — left-assoc set difference.
    let mut terms = p.into_inner();
    let base = lower_term(terms.next().ok_or("empty unary")?, reg, idx)?;
    let mut subs = Vec::new();
    for t in terms {
        subs.push(lower_subtrahend(t, reg, idx)?);
    }
    if subs.is_empty() {
        Ok(base)
    } else {
        Ok(QValue::Diff { base: Box::new(base), subtrahends: subs })
    }
}

/// A subtrahend of `-`: a `{…}` set literal (→ `QValue::Set`) or any other term
/// (a hole / value). Set literals are ONLY valid here, never as a standalone
/// value, so `lower_term` keeps rejecting them.
fn lower_subtrahend(t: Pair<Rule>, reg: &Registry, idx: &HashMap<String, String>) -> Result<QValue, String> {
    match t.as_rule() {
        Rule::set_lit => {
            let mut members = Vec::new();
            for v in t.into_inner() {
                members.push(lower_value(v, reg, idx)?);
            }
            Ok(QValue::Set(members))
        }
        _ => lower_term(t, reg, idx),
    }
}

fn lower_term(p: Pair<Rule>, reg: &Registry, idx: &HashMap<String, String>) -> Result<QValue, String> {
    match p.as_rule() {
        Rule::hole => Ok(QValue::Hole(lower_hole(p)?)),
        Rule::literal => Ok(QValue::Const(p.as_str().to_string())),
        Rule::application => lower_application(p, reg, idx),
        Rule::group => {
            let inner = first_inner(p)?; // group = "(" value ")"
            lower_value(inner, reg, idx)
        }
        Rule::set_lit | Rule::list_lit => {
            Err("set/list literals not supported yet (set-diff / lists phases)".into())
        }
        other => Err(format!("unexpected term {other:?}")),
    }
}

fn lower_hole(p: Pair<Rule>) -> Result<Quant, String> {
    let inner = first_inner(p)?; // hole = sample_n | sample_one | exhaustive
    match inner.as_rule() {
        Rule::exhaustive => Ok(Quant::Exhaustive),
        Rule::sample_one => {
            let seed = match inner.into_inner().next() {
                Some(s) => seed_value(s)?,
                None => 0,
            };
            Ok(Quant::One { seed })
        }
        Rule::sample_n => {
            let mut it = inner.into_inner();
            let n = int_pair(it.next().ok_or("?N needs a count")?)?;
            let seed = match it.next() {
                Some(s) => seed_value(s)?,
                None => 0,
            };
            Ok(Quant::N { n, seed })
        }
        other => Err(format!("unexpected hole {other:?}")),
    }
}

fn lower_application(
    p: Pair<Rule>,
    reg: &Registry,
    idx: &HashMap<String, String>,
) -> Result<QValue, String> {
    // application = ident ~ generic_args?
    let mut it = p.into_inner();
    let head = it.next().ok_or("application needs a head")?.as_str().to_string();
    let gen = it.next();

    let is_wrap = is_wrapper_name(&head);

    // Resolve the head to a catalog component (by type-head), unless it's a
    // wrapper (kept literal, unwrapped by the binding) or an unknown bare ident
    // (a variable reference / nullary the solver resolves from the env).
    let resolved = if is_wrap {
        head.clone()
    } else if let Some(cn) = idx.get(&head) {
        cn.clone()
    } else if gen.is_none() {
        return Ok(QValue::Ident(head)); // var ref or nullary handled by solve
    } else {
        return Err(format!("unknown component `{head}` — no catalog type has that head"));
    };

    let gen = match gen {
        Some(g) => g,
        None => {
            if is_wrap {
                return Err(format!("`{head}<…>` needs a type argument"));
            }
            return Ok(QValue::Ident(resolved)); // nullary component
        }
    };

    // generic_args = "<" arg_list? ">" | "(" arg_list? ")"
    let raw_args: Vec<Pair<Rule>> = match gen.into_inner().next() {
        Some(arg_list) => arg_list.into_inner().collect(),
        None => vec![],
    };

    if is_wrap {
        let mut args = Vec::new();
        for a in raw_args {
            match a.as_rule() {
                Rule::value => args.push(QArg::Pos(lower_value(a, reg, idx)?)),
                other => return Err(format!("`{head}<…>` takes a positional type arg, found {other:?}")),
            }
        }
        return Ok(QValue::App { name: head, args });
    }

    // Component application: named args pass through; positional args bind to the
    // component's declared params in order (skipping ones already named).
    let comp = reg
        .get(&resolved)
        .ok_or_else(|| format!("component `{resolved}` not in catalog"))?;
    let mut named: Vec<String> = Vec::new();
    let mut out: Vec<QArg> = Vec::new();
    let mut positional: Vec<Pair<Rule>> = Vec::new();
    for a in raw_args {
        match a.as_rule() {
            Rule::named_arg => {
                let mut ai = a.into_inner();
                let pname = ai.next().ok_or("named arg needs a name")?.as_str().to_string();
                let pval = lower_value(ai.next().ok_or("named arg needs a value")?, reg, idx)?;
                named.push(pname.clone());
                out.push(QArg::Named { name: pname, value: pval });
            }
            Rule::value => positional.push(a),
            other => return Err(format!("unexpected arg {other:?}")),
        }
    }
    let mut free = comp.params.iter().filter(|pp| !named.contains(&pp.name));
    for a in positional {
        let param = free
            .next()
            .ok_or_else(|| format!("too many positional args for `{resolved}`"))?;
        let pval = lower_value(a, reg, idx)?;
        out.push(QArg::Named { name: param.name.clone(), value: pval });
    }
    Ok(QValue::App { name: resolved, args: out })
}

// ── small pair helpers ──────────────────────────────────────────────────────
fn first_inner(p: Pair<Rule>) -> Result<Pair<Rule>, String> {
    p.into_inner().next().ok_or_else(|| "expected an inner node".to_string())
}
fn int_pair(p: Pair<Rule>) -> Result<usize, String> {
    p.as_str().parse().map_err(|_| format!("bad integer `{}`", p.as_str()))
}
fn seed_value(p: Pair<Rule>) -> Result<u64, String> {
    // seed = "@" ~ integer
    let int = first_inner(p)?;
    int.as_str().parse().map_err(|_| format!("bad seed `{}`", int.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{resolve::resolve, solve::solve, spec::parse_query};

    // The engine self-test catalog (has `shell_sort` with a `seq` slot + the
    // classic/knuth/ciura sequences) — enough to prove the shell slice.
    const REG: &str = include_str!("../registry.spec");

    fn fixture() -> Registry {
        Registry::parse(REG).expect("registry parses")
    }
    fn type_exprs(out: &crate::solve::SolveOutput, reg: &Registry) -> Vec<String> {
        let mut v: Vec<String> = out.sorts.iter().map(|n| resolve(n, reg).unwrap().type_expr).collect();
        v.sort();
        v
    }

    fn ok(src: &str) {
        if let Err(e) = parse(src) {
            panic!("expected `{src}` to parse, got:\n{e}");
        }
    }
    fn err(src: &str) {
        assert!(parse(src).is_err(), "expected `{src}` to FAIL to parse");
    }

    /// Parse `x: T = <src>` and return the flattened list of rule names under the
    /// `value`, for asserting GRAMMAR STRUCTURE of constructs that don't lower
    /// yet (`where`, set-diff, lists).
    fn value_rules(src: &str) -> Vec<String> {
        let prog = format!("x: T = {src}");
        let mut pairs = parse(&prog).unwrap_or_else(|e| panic!("`{src}` should parse:\n{e}"));
        let query = pairs.next().unwrap();
        let binding = query.into_inner().find(|p| p.as_rule() == Rule::binding).unwrap();
        let value = binding.into_inner().find(|p| p.as_rule() == Rule::value).unwrap();
        let mut out = Vec::new();
        collect_rules(value, &mut out);
        out
    }
    fn collect_rules(p: Pair<Rule>, out: &mut Vec<String>) {
        out.push(format!("{:?}", p.as_rule()));
        for c in p.into_inner() {
            collect_rules(c, out);
        }
    }
    fn has(rules: &[String], rule: &str) -> bool {
        rules.iter().any(|r| r == rule)
    }

    // ── each north-star binding parses on its own ──────────────────────────────

    #[test]
    fn simple_shellsorts_binding() {
        ok("ShellSorts: SortingAlgorithm = SortingAlgorithm<ShellSort<_>>");
    }

    #[test]
    fn complex_quicksorts_binding_with_where_union_and_set_diff() {
        ok(r#"
            QuickSorts: _ = SortingAlgorithm<QuickSort<
                partition = _ as p where (
                    StandardPartition<pivot = _ as pp>
                  | DualPivotPartition<pivot = (_ - {Combined<_, _>}) as pp>
                  | HeapExtractionPartition<>
                ),
                pivot     = pp,
                smallSort = _ - {Insertion<16>},
            >>
        "#);
    }

    #[test]
    fn list_typed_binding() {
        ok("Sorts: List<SortingAlgorithm> = [ShellSorts, QuickSorts]");
    }

    #[test]
    fn consumers_over_mains() {
        ok("consumers: List<Mains<Sorts>> = [visualiser, correctness, benchmark]");
    }

    // ── the full program parses as one query ───────────────────────────────────

    #[test]
    fn full_north_star_program() {
        ok(r#"
            // the family of every shell sort
            ShellSorts: SortingAlgorithm = SortingAlgorithm<ShellSort<_>>

            QuickSorts: _ = SortingAlgorithm<QuickSort<
                partition = _ as p where (
                    StandardPartition<pivot = _ as pp>
                  | DualPivotPartition<pivot = (_ - {Combined<_, _>}) as pp>
                  | HeapExtractionPartition<>
                ),
                pivot     = pp,
                smallSort = _ - {Insertion<16>},
            >>

            Sorts:     List<SortingAlgorithm> = [ShellSorts, QuickSorts]
            consumers: List<Mains<Sorts>>     = [visualiser, correctness, benchmark]
        "#);
    }

    // ── feature coverage in isolation ──────────────────────────────────────────

    #[test]
    fn directives_holes_and_intersections() {
        ok("depth 3\nr: RecSort = recursive_sort(inner = _)");
        ok("g: GapSequence = ?2@7"); // random sample of 2, seed 7
        ok("p: Pivot = *"); // star is exhaustive too
        ok("x: Sort & HasTimeBounds & HasStability = ShellSort<Classic>");
        ok("3 of @1\ns: Sort = quick_sort(partition = _, pivot = _, small_sort = _)");
        ok("ins: SmallSort = Insertion<16>"); // positional const
        ok("dual: Pivot = Combined<_, _>"); // positional types
        ok("# comment line\ns: Sort = foo  // trailing\n/* block */");
    }

    // ── malformed inputs are rejected ──────────────────────────────────────────

    #[test]
    fn rejects_malformed() {
        err("foo bar"); // no `: Type = Value`
        err("x: = _"); // missing type annotation
        err("x: Sort ="); // missing value
        err("x: Sort = (unclosed"); // unbalanced paren
        err("x: Sort = SortingAlgorithm<ShellSort<_>"); // unbalanced angle
        err(""); // empty: a query needs at least one binding
    }

    // ── GRAMMAR COVERAGE: the pest parser (acceptance + structure) ─────────────

    #[test]
    fn comments_and_whitespace() {
        ok("# hash comment\nx: T = a");
        ok("// line comment\nx: T = a");
        ok("/* block */ x: T = a");
        ok("x: T = a // trailing\n");
        ok("/* multi\n   line\n   block */\nx: T = a");
        // free-form: a binding spanning many lines parses like a tight one.
        let loose = parse("x : T =\n   Foo<\n     a = _ ,\n     b = _ ,\n   >\n").unwrap();
        let tight = parse("x:T=Foo<a=_,b=_>").unwrap();
        assert_eq!(loose.count(), tight.count());
    }

    #[test]
    fn all_hole_quantifiers_parse() {
        for q in ["_", "*", "?", "?5", "?5@9", "?@9"] {
            ok(&format!("x: T = {q}"));
        }
        // exhaustive vs the sampled forms are recognised as distinct hole rules.
        assert!(has(&value_rules("_"), "exhaustive"));
        assert!(has(&value_rules("?5@9"), "sample_n"));
        assert!(has(&value_rules("?@9"), "sample_one"));
        assert!(has(&value_rules("?5@9"), "seed"));
    }

    #[test]
    fn set_difference_shapes() {
        assert!(has(&value_rules("_ - {a}"), "set_lit"));
        // left-assoc chain: two terms removed
        ok("x: T = _ - {a} - {b}");
        // set with several members + a trailing comma
        ok("x: T = _ - {a, b, c}");
        ok("x: T = _ - {a, b,}");
        // diff inside a group, then captured
        let r = value_rules("(_ - {Combined<_, _>}) as pp");
        assert!(has(&r, "group"));
        assert!(has(&r, "set_lit"));
        assert!(has(&r, "capture"));
    }

    #[test]
    fn where_union_shapes() {
        let r = value_rules("_ as p where ( a | b | c )");
        assert!(has(&r, "capture"));
        assert!(has(&r, "where_clause"));
        // single-branch where is legal
        ok("x: T = _ as p where ( a )");
        // a where whose branches are themselves applications with nested captures
        ok("x: T = _ as p where ( Foo<k = _ as q> | Bar<m = (_ - {z}) as q> )");
    }

    #[test]
    fn lists_and_type_annotations() {
        ok("x: T = []");
        ok("x: T = [a]");
        ok("x: T = [a, b, c]");
        ok("x: T = [a, b,]"); // trailing comma
        assert!(has(&value_rules("[a, b]"), "list_lit"));
        // type annotations: bare role, `_`, generic, nested generic, intersection
        ok("x: SortingAlgorithm = a");
        ok("x: _ = a");
        ok("x: List<SortingAlgorithm> = a");
        ok("x: List<Mains<Sorts>> = a");
        ok("x: Sort & HasTimeBounds & HasStability = a");
    }

    #[test]
    fn generic_arg_forms() {
        ok("x: T = Foo<a = _, b = _>"); // named
        ok("x: T = Foo<_, _>"); // positional
        ok("x: T = Foo<a = _, _>"); // mixed
        ok("x: T = Foo<>"); // empty
        ok("x: T = Foo()"); // paren form
        ok("x: T = Foo(a = _, _)"); // paren + mixed
        ok("x: T = A<B<C<_>>>"); // deep nesting, `>>>` close
        ok("x: T = Foo<a = _,>"); // trailing comma in args
    }

    #[test]
    fn directives_parse_in_position() {
        ok("depth 3\nx: T = a");
        ok("3 of\nx: T = a");
        ok("3 of @7\nx: T = a");
        ok("depth 2\n5 of @1\nx: T = a");
        assert!(has(
            &{
                let mut pairs = parse("depth 3\nx: T = a").unwrap();
                let q = pairs.next().unwrap();
                let mut v = Vec::new();
                for p in q.into_inner() {
                    v.push(format!("{:?}", p.as_rule()));
                }
                v
            },
            "depth_dir"
        ));
    }

    #[test]
    fn more_malformed_inputs_rejected() {
        err("x: T = a where ( )"); // empty where
        err("x: T = a where ( a | )"); // dangling pipe
        err("x: T = _ as"); // capture without a name
        err("x: T = _ -"); // dangling minus
        err("x: T = {a, b"); // unbalanced brace
        err("x: T = [a, b"); // unbalanced bracket
        err("x: T = Foo<a = >"); // arg without a value
        err("x = a"); // missing `: Type`
        err("depth\nx: T = a"); // depth without a number
    }

    // ── STEP 1: the shell slice — lower onto the existing AST + solver ──────────

    #[test]
    fn wrapper_and_positional_lowering() {
        let reg = fixture();
        let qs =
            lower_program("X: SortingAlgorithm = SortingAlgorithm<ShellSort<_>>", &reg).unwrap();
        assert_eq!(qs.len(), 1);
        let b = &qs[0].bindings[0];
        assert_eq!(b.role, "Sort"); // from the SortingAlgorithm wrapper
        match &b.value {
            QValue::App { name, args } => {
                assert_eq!(name, "shell_sort"); // resolved by type-head (ShellSort)
                assert_eq!(args.len(), 1);
                match &args[0] {
                    QArg::Named { name, value } => {
                        assert_eq!(name, "seq"); // positional `_` → the seq slot
                        assert!(matches!(value, QValue::Hole(Quant::Exhaustive)));
                    }
                    other => panic!("expected named seq arg, got {other:?}"),
                }
            }
            other => panic!("expected App, got {other:?}"),
        }
    }

    /// The headline: the new front-end reproduces the existing query's output.
    #[test]
    fn shell_slice_matches_existing_query() {
        let reg = fixture();
        let new_qs =
            lower_program("ShellSorts: SortingAlgorithm = SortingAlgorithm<ShellSort<_>>", &reg)
                .unwrap();
        let new_out = solve(&new_qs[0], &reg).unwrap();

        let old_out = solve(&parse_query("let s: Sort = shell_sort(seq = .);").unwrap(), &reg).unwrap();

        assert_eq!(type_exprs(&new_out, &reg), type_exprs(&old_out, &reg));
        let exprs = type_exprs(&new_out, &reg);
        assert_eq!(exprs.len(), 3); // classic, knuth, ciura
        assert!(exprs.iter().all(|e| e.starts_with("ShellSort<")));
    }

    #[test]
    fn explicit_named_sequence_resolves_by_type_head() {
        let reg = fixture();
        let qs =
            lower_program("X: SortingAlgorithm = SortingAlgorithm<ShellSort<Classic>>", &reg)
                .unwrap();
        let out = solve(&qs[0], &reg).unwrap();
        assert_eq!(type_exprs(&out, &reg), vec!["ShellSort<Classic>".to_string()]);
    }

    #[test]
    fn random_quantifier_lowers() {
        let reg = fixture();
        let qs = lower_program("X: SortingAlgorithm = SortingAlgorithm<ShellSort<?2@7>>", &reg)
            .unwrap();
        let out = solve(&qs[0], &reg).unwrap();
        assert_eq!(out.sorts.len(), 2); // 2 sampled sequences, seeded
    }

    #[test]
    fn unsupported_constructs_error_clearly() {
        let reg = fixture();
        assert!(lower_program("X: _ = ShellSort<_>", &reg).is_err()); // can't infer role from `_`
        // a `{…}` set literal is only meaningful as a subtrahend after `-`.
        assert!(lower_program("p: Pivot = {first_element}", &reg).is_err());
        // lists are a separate, later track.
        assert!(lower_program("xs: List<Pivot> = [first_element]", &reg).is_err());
    }

    // ── STEP 4: set difference (`_ - {X}`) ─────────────────────────────────────

    #[test]
    fn set_diff_lowers_to_diff_node() {
        let reg = fixture();
        let qs = lower_program("P: Pivot = _ - {middle_element}", &reg).unwrap();
        match &qs[0].bindings[0].value {
            QValue::Diff { base, subtrahends } => {
                assert_eq!(**base, QValue::Hole(Quant::Exhaustive));
                assert_eq!(subtrahends.len(), 1);
                assert_eq!(subtrahends[0], QValue::Set(vec![QValue::Ident("middle_element".into())]));
            }
            other => panic!("expected Diff, got {other:?}"),
        }
    }

    #[test]
    fn set_diff_removes_named_members() {
        let reg = fixture();
        // 7 Pivot providers (first, middle, ninther, combined×4) minus middle = 6.
        let qs = lower_program("P: Pivot = _ - {middle_element}", &reg).unwrap();
        let exprs = type_exprs(&solve(&qs[0], &reg).unwrap(), &reg);
        assert_eq!(exprs.len(), 6);
        assert!(!exprs.contains(&"MiddleElement".to_string()));
        assert!(exprs.contains(&"FirstElement".to_string()));
    }

    #[test]
    fn set_diff_multiple_members_and_chaining_agree() {
        let reg = fixture();
        let multi = lower_program("P: Pivot = _ - {first_element, middle_element}", &reg).unwrap();
        let chain = lower_program("P: Pivot = _ - {first_element} - {middle_element}", &reg).unwrap();
        let m = type_exprs(&solve(&multi[0], &reg).unwrap(), &reg);
        let c = type_exprs(&solve(&chain[0], &reg).unwrap(), &reg);
        assert_eq!(m, c); // `{a, b}` and `- {a} - {b}` are the same
        assert_eq!(m.len(), 5); // 7 - 2
        assert!(!m.contains(&"FirstElement".to_string()));
        assert!(!m.contains(&"MiddleElement".to_string()));
    }

    #[test]
    fn set_diff_subtracting_all_is_empty() {
        let reg = fixture();
        // `_ - _` = exhaustive minus exhaustive = ∅  (the HeapExtraction trick).
        let qs = lower_program("P: Pivot = _ - _", &reg).unwrap();
        assert_eq!(solve(&qs[0], &reg).unwrap().sorts.len(), 0);
    }

    #[test]
    fn set_diff_removes_a_specific_nested_shape() {
        let reg = fixture();
        // remove exactly one combined pair, leaving 7 - 1 = 6.
        let qs = lower_program(
            "P: Pivot = _ - {CombinedSelector<a = first_element, b = first_element>}",
            &reg,
        )
        .unwrap();
        let exprs = type_exprs(&solve(&qs[0], &reg).unwrap(), &reg);
        assert_eq!(exprs.len(), 6);
        assert!(!exprs.contains(&"CombinedSelector<FirstElement, FirstElement>".to_string()));
        assert!(exprs.contains(&"CombinedSelector<MiddleElement, MiddleElement>".to_string()));
    }

    #[test]
    fn set_diff_composes_with_capture() {
        let reg = fixture();
        // slot `a` ranges over PivotSingle (first, middle) minus middle = {first},
        // captured as x; b reuses it → only combined<first, first>.
        let qs = lower_program(
            "P: Pivot = CombinedSelector<a = _ - {middle_element} as x, b = x>",
            &reg,
        )
        .unwrap();
        let exprs = type_exprs(&solve(&qs[0], &reg).unwrap(), &reg);
        assert_eq!(exprs, vec!["CombinedSelector<FirstElement, FirstElement>".to_string()]);
    }

    // ── STEP 5b: set difference composes with sampling + recursive roles ───────

    /// Subtraction happens BEFORE the quantifier samples: `?2 - {first}` always
    /// returns 2 of the 6-element complement (never `first`), for every seed.
    /// (The old order sampled 2 from all 7, then removed `first` → sometimes 1.)
    #[test]
    fn set_diff_subtracts_before_sampling() {
        let reg = fixture();
        for seed in 0..8u64 {
            let qs =
                lower_program(&format!("P: Pivot = ?2@{seed} - {{first_element}}"), &reg).unwrap();
            let exprs = type_exprs(&solve(&qs[0], &reg).unwrap(), &reg);
            assert_eq!(exprs.len(), 2, "seed {seed}");
            assert!(!exprs.contains(&"FirstElement".to_string()), "seed {seed}");
        }
    }

    /// Set difference over a RECURSIVE role: the complement of `BaseCase` within
    /// the depth-3 `RecSort` family (4 trees) is the other 3.
    #[test]
    fn set_diff_over_a_recursive_role() {
        let reg = fixture();
        let qs = lower_program("depth 3\nr: RecSort = _ - {BaseCase}", &reg).unwrap();
        let exprs = type_exprs(&solve(&qs[0], &reg).unwrap(), &reg);
        assert_eq!(exprs.len(), 3);
        assert!(!exprs.contains(&"BaseCase".to_string()));
    }

    // ── STEP 5 PROBES: cross-branch capture (characterize current behaviour) ────
    //
    // registry.spec's partitions are nullary, so it can't express "a variable
    // captured INSIDE a union branch and referenced OUTSIDE". This minimal probe
    // catalog can: `Box<main, echo>` references a leaf captured inside the
    // `main` union at the sibling `echo` slot.
    const XCAT: &str = "\
component box
  type Box<{main}, {echo}>
  provides Top
  slot main Inner
  slot echo Leaf
end
component wrap_a
  type WrapA<{x}>
  provides Inner
  slot x Leaf
end
component wrap_b
  type WrapB<{x}>
  provides Inner
  slot x Leaf
end
component wrap_none
  type WrapNone
  provides Inner
end
component leaf1
  type Leaf1
  provides Leaf
end
component leaf2
  type Leaf2
  provides Leaf
end
";

    fn xcat() -> Registry {
        Registry::parse(XCAT).expect("probe catalog parses")
    }

    /// WORKS TODAY: a variable captured inside two different union branches and
    /// referenced at a sibling slot couples correctly — `echo` always matches the
    /// leaf chosen inside `main`. 4 results, not 8 (no spurious mismatches).
    #[test]
    fn probe_xbranch_capture_couples_across_branches() {
        let reg = xcat();
        let qs = lower_program(
            "b: Top = Box< main = _ where ( WrapA<x = _ as shared> | WrapB<x = _ as shared> ), echo = shared >",
            &reg,
        )
        .unwrap();
        let exprs = type_exprs(&solve(&qs[0], &reg).unwrap(), &reg);
        assert_eq!(
            exprs,
            vec![
                "Box<WrapA<Leaf1>, Leaf1>".to_string(),
                "Box<WrapA<Leaf2>, Leaf2>".to_string(),
                "Box<WrapB<Leaf1>, Leaf1>".to_string(),
                "Box<WrapB<Leaf2>, Leaf2>".to_string(),
            ]
        );
    }

    /// WORKS TODAY: a set-difference whose subtrahend references an
    /// earlier-bound capture resolves per-candidate (`echo` = the OTHER leaf).
    #[test]
    fn probe_set_diff_subtrahend_references_earlier_capture() {
        let reg = xcat();
        let qs = lower_program(
            "b: Top = Box< main = WrapA<x = _ as shared>, echo = _ - {shared} >",
            &reg,
        )
        .unwrap();
        let exprs = type_exprs(&solve(&qs[0], &reg).unwrap(), &reg);
        assert_eq!(
            exprs,
            vec!["Box<WrapA<Leaf1>, Leaf2>".to_string(), "Box<WrapA<Leaf2>, Leaf1>".to_string()]
        );
    }

    /// G1 FIX: a union branch that does NOT bind `shared` (WrapNone) is PRUNED
    /// (yields ∅ for that candidate), not an error — the valid WrapA results
    /// survive. This is the north-star's empty `HeapExtraction` branch.
    #[test]
    fn unbound_branch_reference_is_pruned() {
        let reg = xcat();
        let qs = lower_program(
            "b: Top = Box< main = _ where ( WrapA<x = _ as shared> | WrapNone ), echo = shared >",
            &reg,
        )
        .unwrap();
        let exprs = type_exprs(&solve(&qs[0], &reg).unwrap(), &reg);
        assert_eq!(
            exprs,
            vec!["Box<WrapA<Leaf1>, Leaf1>".to_string(), "Box<WrapA<Leaf2>, Leaf2>".to_string()]
        );
    }

    /// Forward reference (`echo` uses `shared` before `main` binds it) is a CLEAR
    /// error — not a silent drop. Forward/cyclic refs are out of scope (the lazy
    /// 5b track is about recursive complements, not ordering).
    #[test]
    fn forward_reference_is_a_clear_error() {
        let reg = xcat();
        let qs = lower_program(
            "b: Top = Box< echo = shared, main = WrapA<x = _ as shared> >",
            &reg,
        )
        .unwrap();
        let err = solve(&qs[0], &reg).unwrap_err();
        assert!(err.contains("before it is captured"), "got: {err}");
    }

    /// Same clear error for a forward reference inside a subtrahend.
    #[test]
    fn forward_reference_in_subtrahend_is_a_clear_error() {
        let reg = xcat();
        let qs = lower_program(
            "b: Top = Box< echo = _ - {shared}, main = WrapA<x = _ as shared> >",
            &reg,
        )
        .unwrap();
        let err = solve(&qs[0], &reg).unwrap_err();
        assert!(err.contains("before it is captured"), "got: {err}");
    }

    // ── STEP 3: `where ( A | B | C )` unions ───────────────────────────────────

    #[test]
    fn where_union_lowers_to_where_node() {
        let reg = fixture();
        let qs =
            lower_program("P: Partition = _ where ( LeftLeftPartition | DualPivotPartition )", &reg)
                .unwrap();
        match &qs[0].bindings[0].value {
            QValue::Where { quant, alts } => {
                assert_eq!(*quant, Quant::Exhaustive); // from the `_` base
                assert_eq!(alts.len(), 2);
                // alternatives resolved by type-head to their components
                assert_eq!(alts[0], QValue::Ident("LL_partition".into()));
                assert_eq!(alts[1], QValue::Ident("dual_pivot_partition".into()));
            }
            other => panic!("expected Where, got {other:?}"),
        }
    }

    /// A union is REPLACEMENT, not "all-providers-then-filter": the population is
    /// exactly the alternatives' expansions (here `first` ∪ `combined<a,b>` = 5),
    /// NOT every Pivot provider (which would also include middle / ninther).
    #[test]
    fn where_union_is_the_union_of_alternatives() {
        let reg = fixture();
        let qs = lower_program(
            "P: Pivot = _ where ( first_element | CombinedSelector<a = _, b = _> )",
            &reg,
        )
        .unwrap();
        let out = solve(&qs[0], &reg).unwrap();
        let exprs = type_exprs(&out, &reg);
        assert_eq!(exprs.len(), 5); // first(1) + combined a×b(4)
        assert!(exprs.contains(&"FirstElement".to_string()));
        assert!(!exprs.contains(&"MiddleElement".to_string())); // not in the union
        assert!(exprs.iter().filter(|e| e.starts_with("CombinedSelector<")).count() == 4);
    }

    #[test]
    fn where_union_respects_the_base_quantifier() {
        let reg = fixture();
        let qs = lower_program(
            "P: Pivot = ?2@1 where ( first_element | middle_element | ninther_dual )",
            &reg,
        )
        .unwrap();
        let out = solve(&qs[0], &reg).unwrap();
        assert_eq!(out.sorts.len(), 2); // sampled 2 of the 3 alternatives
    }

    /// The union RESULT can be captured and reused (step-2 capture over a Where):
    /// `a` picks from the restricted population and `b` reuses it → the diagonal.
    #[test]
    fn where_union_result_can_be_captured() {
        let reg = fixture();
        let qs = lower_program(
            "P: Pivot = CombinedSelector<a = _ as x where ( first_element | middle_element ), b = x>",
            &reg,
        )
        .unwrap();
        let out = solve(&qs[0], &reg).unwrap();
        let exprs = type_exprs(&out, &reg);
        assert_eq!(exprs.len(), 2); // diagonal over the 2-element union
        assert!(exprs.iter().all(|t| {
            let inner = t.trim_start_matches("CombinedSelector<").trim_end_matches('>');
            let (a, b) = inner.split_once(", ").unwrap();
            a == b
        }));
    }

    #[test]
    fn where_requires_a_hole_base() {
        let reg = fixture();
        // a non-hole base before `where` is rejected (for now).
        assert!(lower_program("P: Pivot = first_element where ( middle_element )", &reg).is_err());
    }

    // ── STEP 2: capture (`as p`) ───────────────────────────────────────────────

    #[test]
    fn capture_lowers_to_capture_node() {
        let reg = fixture();
        let qs = lower_program(
            "P: Pivot = CombinedSelector<a = _ as x, b = x>",
            &reg,
        )
        .unwrap();
        let b = &qs[0].bindings[0];
        match &b.value {
            QValue::App { name, args } => {
                assert_eq!(name, "combined"); // resolved by type-head
                // arg a is `_ as x` → a Capture wrapping a hole
                match &args[0] {
                    QArg::Named { name, value } => {
                        assert_eq!(name, "a");
                        assert!(matches!(value, QValue::Capture { name, .. } if name == "x"));
                    }
                    other => panic!("expected named `a`, got {other:?}"),
                }
                // arg b is a bare reference to x
                match &args[1] {
                    QArg::Named { name, value } => {
                        assert_eq!(name, "b");
                        assert!(matches!(value, QValue::Ident(n) if n == "x"));
                    }
                    other => panic!("expected named `b`, got {other:?}"),
                }
            }
            other => panic!("expected App, got {other:?}"),
        }
    }

    /// Sibling-arg capture: `a` captures `x`, `b` reuses it → the DIAGONAL
    /// (a == b), not the full a×b cross-product. This is the capture mechanism
    /// (propagation + unify) working end-to-end through the solver.
    #[test]
    fn sibling_capture_yields_the_diagonal() {
        let reg = fixture();
        let captured = lower_program("P: Pivot = CombinedSelector<a = _ as x, b = x>", &reg).unwrap();
        let cap_out = solve(&captured[0], &reg).unwrap();
        let cap = type_exprs(&cap_out, &reg);

        // For contrast, the un-captured cross-product over both single pivots.
        let cross = lower_program("P: Pivot = CombinedSelector<a = _, b = _>", &reg).unwrap();
        let cross_out = solve(&cross[0], &reg).unwrap();

        // registry.spec has 2 single pivots (first, middle): diagonal = 2, cross = 4.
        assert_eq!(cap.len(), 2);
        assert_eq!(cross_out.sorts.len(), 4);
        // every captured result has a == b (the same pivot on both sides)
        assert!(cap.iter().all(|t| {
            let inner = t.trim_start_matches("CombinedSelector<").trim_end_matches('>');
            let (a, b) = inner.split_once(", ").unwrap();
            a == b
        }));
    }

    /// Capture escaping to a LATER binding: `shared` is captured deep inside the
    /// first binding's value and referenced by the second (the target). The flat
    /// program env carries it across the binding boundary.
    #[test]
    fn capture_escapes_to_a_later_binding() {
        let reg = fixture();
        // Built directly (lower_program splits emittable bindings into separate
        // queries; cross-binding feeders are a later lowering step). This tests
        // the SOLVER's capture-escape + unify across bindings.
        let q = Query {
            depth: DEFAULT_DEPTH,
            take: None,
            bindings: vec![
                // x: Pivot = CombinedSelector<a = _ as shared, b = first_element>
                Binding {
                    name: "x".into(),
                    role: "Pivot".into(),
                    refinements: vec![],
                    value: QValue::App {
                        name: "combined".into(),
                        args: vec![
                            QArg::Named {
                                name: "a".into(),
                                value: QValue::Capture {
                                    name: "shared".into(),
                                    inner: Box::new(QValue::Hole(Quant::Exhaustive)),
                                },
                            },
                            QArg::Named {
                                name: "b".into(),
                                value: QValue::Ident("first_element".into()),
                            },
                        ],
                    },
                },
                // y: Pivot = shared   (the target references the captured var)
                Binding {
                    name: "y".into(),
                    role: "Pivot".into(),
                    refinements: vec![],
                    value: QValue::Ident("shared".into()),
                },
            ],
        };
        let out = solve(&q, &reg).unwrap();
        // `shared` ranges over the 2 single pivots; y returns exactly those.
        let exprs = type_exprs(&out, &reg);
        assert_eq!(exprs, vec!["FirstElement".to_string(), "MiddleElement".to_string()]);
    }

    // ── consumers / List / Mains: whole-program lowering ───────────────────────

    #[test]
    fn lowers_a_full_consumers_program() {
        let reg = fixture();
        let prog = lower(
            r#"
            ShellSorts: SortingAlgorithm = SortingAlgorithm<ShellSort<_>>
            Sorts:      List<SortingAlgorithm> = [ShellSorts]
            consumers:  List<Mains<Sorts>> = [visualiser, correctness, benchmark]
            "#,
            &reg,
        )
        .unwrap();

        assert_eq!(prog.algos.len(), 1);
        assert_eq!(prog.algos[0].name, "ShellSorts");
        assert_eq!(
            prog.sets,
            vec![NamedSet { name: "Sorts".into(), members: vec!["ShellSorts".into()] }]
        );
        assert_eq!(
            prog.consumers,
            vec![ConsumerDecl {
                name: "consumers".into(),
                target: "Sorts".into(),
                mains: vec!["visualiser".into(), "correctness".into(), "benchmark".into()],
            }]
        );
    }

    #[test]
    fn resolve_algos_flattens_sets_transitively() {
        let reg = fixture();
        let prog = lower(
            r#"
            A:     SortingAlgorithm = SortingAlgorithm<ShellSort<Classic>>
            B:     SortingAlgorithm = SortingAlgorithm<ShellSort<Knuth>>
            Inner: List<SortingAlgorithm> = [A]
            Sorts: List<SortingAlgorithm> = [Inner, B]
            consumers: List<Mains<Sorts>> = [correctness]
            "#,
            &reg,
        )
        .unwrap();
        // Sorts → [Inner → [A], B] → [A, B]
        assert_eq!(prog.resolve_algos("Sorts").unwrap(), vec!["A".to_string(), "B".to_string()]);
        assert_eq!(prog.resolve_algos("A").unwrap(), vec!["A".to_string()]);
        assert!(prog.resolve_algos("missing").is_err());
    }

    /// The bridge to the runtime: each consumer resolves to the concrete sort
    /// LABELS (matching the emitted `AlgorithmEntry` names) it should run over.
    #[test]
    fn resolve_consumers_yields_concrete_labels() {
        let reg = fixture();
        let prog = lower(
            r#"
            ShellSorts: SortingAlgorithm = SortingAlgorithm<ShellSort<_>>
            Sorts:      List<SortingAlgorithm> = [ShellSorts]
            consumers:  List<Mains<Sorts>> = [visualiser, correctness]
            "#,
            &reg,
        )
        .unwrap();
        let resolved = prog.resolve_consumers(&reg).unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].name, "consumers");
        assert_eq!(resolved[0].mains, vec!["visualiser".to_string(), "correctness".to_string()]);
        // the fixture has 3 gap sequences → 3 distinct shell labels (`shell[…]`).
        assert_eq!(resolved[0].algo_labels.len(), 3);
        assert!(resolved[0].algo_labels.iter().all(|l| l.starts_with("shell[")));
    }

    #[test]
    fn lower_rejects_duplicate_binding_names() {
        let reg = fixture();
        let err = lower(
            r#"
            A: SortingAlgorithm = SortingAlgorithm<ShellSort<Classic>>
            A: SortingAlgorithm = SortingAlgorithm<ShellSort<Knuth>>
            "#,
            &reg,
        )
        .unwrap_err();
        assert!(err.contains("duplicate binding name `A`"), "got: {err}");
    }

    // ── refinement syntax (`Role[param = value]`) ─────────────────────────────

    #[test]
    fn refinement_syntax_parses_and_lowers() {
        let reg = fixture();
        ok("part: Partition[pivot = p] = _");
        ok("part: Partition[pivot = p, small = q] = _"); // multiple refinements
        ok("part: Partition[pivot = p,] = _"); // trailing comma

        let qs = lower_program("part: Partition[pivot = p] = _", &reg).unwrap();
        let b = &qs[0].bindings[0];
        assert_eq!(b.role, "Partition");
        assert_eq!(
            b.refinements,
            vec![crate::spec::Refinement {
                param: "pivot".into(),
                value: QValue::Ident("p".into()),
            }]
        );
        assert_eq!(b.value, QValue::Hole(Quant::Exhaustive));
    }

    #[test]
    fn helper_bindings_accumulate_into_the_family_query() {
        let reg = fixture();
        let prog = lower(
            r#"
            p:    Pivot = _
            part: Partition[pivot = p] = _
            QuickSorts: SortingAlgorithm = SortingAlgorithm<QuickSort<partition = part, pivot = p, small_sort = _>>
            "#,
            &reg,
        )
        .unwrap();
        assert_eq!(prog.algos.len(), 1);
        let q = &prog.algos[0].query;
        // the two helpers are prepended before the emittable binding → 3 bindings.
        assert_eq!(q.bindings.len(), 3);
        assert_eq!(q.bindings[0].name, "p");
        assert_eq!(q.bindings[1].name, "part");
        assert_eq!(q.bindings[2].name, "QuickSorts");
        // solving reproduces the proven shared-pivot arity coupling (the fixture's
        // `shared_pivot_makes_arity_mismatch_unrepresentable`): 7 pivots, each with
        // its arity-matching partition, × 3 small sorts = 21.
        let out = crate::solve::solve(q, &reg).unwrap();
        assert_eq!(out.sorts.len(), 21);
    }

    #[test]
    fn helpers_are_shared_across_multiple_families() {
        let reg = fixture();
        // one `p`/`part` reused by two families: each family's Query gets both
        // helpers prepended (3 bindings each), no duplicate-name error.
        let prog = lower(
            r#"
            p:    Pivot = _
            part: Partition[pivot = p] = _
            A: SortingAlgorithm = SortingAlgorithm<QuickSort<partition = part, pivot = p, small_sort = _>>
            B: SortingAlgorithm = SortingAlgorithm<QuickSort<partition = part, pivot = p, small_sort = no_small_sort>>
            "#,
            &reg,
        )
        .unwrap();
        assert_eq!(prog.algos.len(), 2);
        assert_eq!(prog.algos[0].query.bindings.len(), 3);
        assert_eq!(prog.algos[1].query.bindings.len(), 3);
    }

    #[test]
    fn lower_program_still_rejects_lists() {
        // `lower_program` (the per-binding Vec<Query> entry point) is unchanged:
        // lists/consumers are only lowered by `lower`.
        let reg = fixture();
        assert!(lower_program("xs: List<Pivot> = [first_element]", &reg).is_err());
    }
}
