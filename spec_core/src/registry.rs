//! Stage 1 — the text catalog: parse `registry.spec` into a component graph.

use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Clone)]
pub enum ParamKind {
    /// Filled by a nested component that `provides` `role`. A query must fill it.
    Type { role: String },
    /// Filled by a literal token (integer OR `true`/`false`; real const
    /// generics include `const N: usize` and `const PING_PONG: bool`). Matched
    /// either by name (`ping_pong = true`) or positionally (`insertion<32>`).
    /// A query must supply it. `values` is the declared finite set of "neat
    /// values" a quantified const hole (`threshold = *`/`?`) ranges over,
    /// membership only, never arithmetic. Empty = the const is literal only and
    /// a hole over it is an error.
    Const { values: Vec<String> },
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub kind: ParamKind,
}

#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub type_tmpl: String,
    pub label_tmpl: String,
    /// The base roles this component joins, from its `provides` line. A derived
    /// role is defined by a `role` line, so ask the registry (`has_role`,
    /// `roles_of`) rather than reading this for membership.
    pub provides: Vec<String>,
    pub params: Vec<Param>,
    /// Module paths to bring into scope for `type_tmpl` to resolve, e.g.
    /// `quick_sort_lib::Pivoted`. Empty means the imports are derived from the
    /// owning crate; a listed `uses` replaces that derivation. Unioned with
    /// nested children's imports during resolve.
    pub uses: Vec<String>,
    /// The crate whose fragment declared this component, when the registry was
    /// built from fragments. Imports are derived from it unless `uses` overrides.
    pub krate: Option<String>,
    /// Per-family literal (NOT compositional — can't be inherited from the
    /// type): true if the algorithm runs faster on nearly-sorted input.
    pub adaptive: bool,
    /// Per-family literal: true if the algorithm's `SortLog` trace is
    /// intentionally nondeterministic between runs (e.g. randomised gaps). The
    /// emit backend copies it into `AlgorithmEntry.nondeterministic`.
    pub nondeterministic: bool,
    /// Contract-defined upper bound on input size → `AlgorithmEntry.max_input_size`.
    pub max_input: Option<usize>,
    /// Structural picker sub-path beneath the category root, parsed from a
    /// `menu a / b / c` line (slash-separated, each segment trimmed). Empty =
    /// register flat at the root. Read by `emit_entries` on a query's ROOT
    /// component; navigation placement only, never affects dispatch.
    pub menu: Vec<String>,
}

/// The leading identifier of a `type` template, e.g. `ShellSort<{seq}>` gives `ShellSort`.
pub fn type_head(type_tmpl: &str) -> &str {
    let t = type_tmpl.trim();
    let end = t.find(|c: char| c == '<' || c.is_whitespace()).unwrap_or(t.len());
    &t[..end]
}

impl Component {
    /// The Rust type a query names this component by.
    pub fn head(&self) -> &str {
        type_head(&self.type_tmpl)
    }

    /// The path the head is imported from: the `uses` entry that ends in it when
    /// `uses` is listed, else the head at the root of the owning crate.
    pub fn head_path(&self) -> Option<String> {
        let head = self.head();
        if self.uses.is_empty() {
            return self.krate.as_ref().map(|krate| format!("{krate}::{head}"));
        }
        self.uses.iter().find(|u| u.rsplit("::").next() == Some(head)).cloned()
    }

    /// Who declared the component, as an error message names it.
    fn declarer(&self) -> String {
        declarer(self.krate.as_deref())
    }

    /// A referenceable param by name, a type slot or a const. A refinement
    /// `[name = ...]` is only valid if this returns `Some`.
    pub fn param(&self, name: &str) -> Option<&Param> {
        self.params.iter().find(|p| p.name == name)
    }

    /// The role a referenceable param contributes to whatever is bound to it.
    /// `Type` carries a role; `Const` carries none (its constraint is the value
    /// set, not a role).
    pub fn param_role(&self, name: &str) -> Option<&str> {
        match &self.param(name)?.kind {
            ParamKind::Type { role } => Some(role),
            ParamKind::Const { .. } => None,
        }
    }
}

/// Who wrote a definition, as an error message names it.
fn declarer(krate: Option<&str>) -> String {
    krate.map_or_else(|| "an unowned fragment".to_string(), |krate| format!("`{krate}`"))
}

/// What a term of a `role` line does to the members gathered so far.
#[derive(Debug, Clone, Copy)]
enum Op {
    Add,
    Remove,
}

/// A role defined by a `role` line as other roles and components added and
/// removed, rather than tagged on each member's `provides` line.
#[derive(Debug)]
struct DerivedRole {
    /// The crate whose fragment wrote the line.
    krate: Option<String>,
    /// The terms as written, a role name or a component name each. The first adds.
    terms: Vec<(Op, String)>,
    /// The member component names, sorted. Empty until `resolve_roles` has run
    /// over the whole registry, because a term may name a component of any fragment.
    members: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Registry {
    components: HashMap<String, Component>,
    /// Component names in PARSE (insertion) order. `iter()` follows this so
    /// type-head indexing is deterministic and first-wins-on-collision resolves
    /// to the component declared FIRST (e.g. the generic `combined` /
    /// `CombinedSelector<{a},{b}>` before beap's concrete `CombinedSelector<…>`
    /// instantiations; `pk_single` before `pk_dual`). A raw `HashMap` iteration
    /// would make head resolution — and thus the emitted catalog — nondeterministic.
    order: Vec<String>,
    /// Type head -> the first component declared with it.
    heads: HashMap<String, String>,
    /// Derived role name -> its definition. A role is base (on `provides` lines)
    /// or derived (here), never both.
    derived: BTreeMap<String, DerivedRole>,
}

impl Registry {
    /// Insert a component, recording its parse position for ordered iteration.
    /// Two components may share a type head only when the head is imported from
    /// one path, because a head names exactly one Rust type, in a query and in
    /// the emitted `use` block.
    fn insert_component(&mut self, c: Component) -> Result<(), String> {
        if let Some(prev) = self.components.get(&c.name) {
            return Err(format!(
                "duplicate component `{}`, declared by {} and {}; \
                 names are unique across the whole catalog",
                c.name,
                prev.declarer(),
                c.declarer()
            ));
        }
        let clash = self.iter().find(|o| {
            let same_head = o.name != c.name && o.head() == c.head();
            same_head && matches!((o.head_path(), c.head_path()), (Some(a), Some(b)) if a != b)
        });
        if let Some(o) = clash {
            return Err(format!(
                "`{}` and `{}` share the type head `{}` but import it from `{}` and `{}`; \
                 type heads are unique workspace wide, rename one of the Rust types",
                o.name,
                c.name,
                c.head(),
                o.head_path().unwrap_or_default(),
                c.head_path().unwrap_or_default()
            ));
        }
        self.order.push(c.name.clone());
        self.heads.entry(c.head().to_string()).or_insert_with(|| c.name.clone());
        self.components.insert(c.name.clone(), c);
        Ok(())
    }

    /// Record a derived role, once per name across everything merged in.
    fn insert_derived(&mut self, name: String, role: DerivedRole) -> Result<(), String> {
        if let Some(prev) = self.derived.get(&name) {
            return Err(format!(
                "duplicate derived role `{name}`, defined by {} and {}; \
                 a derived role has one definition across the whole catalog",
                declarer(prev.krate.as_deref()),
                declarer(role.krate.as_deref())
            ));
        }
        self.derived.insert(name, role);
        Ok(())
    }

    /// Parse a text whose components have no owning crate, so each must list `uses`.
    /// Every term of a `role` line must resolve within the text.
    pub fn parse(text: &str) -> Result<Registry, String> {
        let mut reg = Self::parse_owned(text, None)?;
        reg.resolve_roles()?;
        Ok(reg)
    }

    /// Parse the fragment owned by crate `krate`; its components import from it.
    /// Every term of a `role` line must resolve within the fragment, so a fragment
    /// that derives a role from another crate's components goes through
    /// `from_fragments`.
    pub fn parse_fragment(text: &str, krate: &str) -> Result<Registry, String> {
        let mut reg = Self::parse_owned(text, Some(krate))?;
        reg.resolve_roles()?;
        Ok(reg)
    }

    /// Move every component and derived role of `other` in after this registry's own.
    fn merge(&mut self, other: Registry) -> Result<(), String> {
        let Registry { mut components, order, derived, .. } = other;
        for name in order {
            self.insert_component(components.remove(&name).expect("order names a component"))?;
        }
        derived.into_iter().try_for_each(|(name, role)| self.insert_derived(name, role))
    }

    /// One registry over every `(crate, fragment text)`, in the order given. The
    /// derived roles are resolved over the merged whole, so a `role` line may name
    /// the components and roles of any fragment.
    pub fn from_fragments<'a>(
        fragments: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Registry, String> {
        let mut reg = Registry::default();
        for (krate, text) in fragments {
            let fragment = Registry::parse_owned(text, Some(krate))
                .map_err(|e| format!("crate `{krate}`: {e}"))?;
            reg.merge(fragment)?;
        }
        reg.resolve_roles()?;
        Ok(reg)
    }

    fn parse_owned(text: &str, krate: Option<&str>) -> Result<Registry, String> {
        let mut reg = Registry::default();
        let mut cur: Option<Component> = None;
        for (lineno, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (kw, rest) = split_first_word(line);
            let err = |m: &str| format!("registry line {}: {m}", lineno + 1);
            match kw {
                "role" => {
                    if let Some(c) = &cur {
                        return Err(err(&format!(
                            "`role` inside component `{}`, put it between components",
                            c.name
                        )));
                    }
                    let (name, role) = parse_role(rest, krate).map_err(|m| err(&m))?;
                    reg.insert_derived(name, role).map_err(|m| err(&m))?;
                }
                "component" => {
                    if let Some(c) = cur.take() {
                        reg.insert_component(c).map_err(|m| err(&m))?;
                    }
                    cur = Some(Component {
                        name: rest.trim().to_string(),
                        type_tmpl: String::new(),
                        label_tmpl: String::new(),
                        provides: vec![],
                        params: vec![],
                        uses: vec![],
                        krate: krate.map(str::to_string),
                        adaptive: false,
                        nondeterministic: false,
                        max_input: None,
                        menu: vec![],
                    });
                }
                "end" => {
                    let c = cur.take().ok_or_else(|| err("`end` outside a component"))?;
                    reg.insert_component(c).map_err(|m| err(&m))?;
                }
                _ => {
                    let c = cur.as_mut().ok_or_else(|| err("field outside a component"))?;
                    match kw {
                        "type" => c.type_tmpl = rest.trim().to_string(),
                        "label" => c.label_tmpl = rest.trim().to_string(),
                        "provides" => {
                            c.provides = rest.split_whitespace().map(str::to_string).collect()
                        }
                        "uses" => {
                            c.uses.extend(rest.split_whitespace().map(str::to_string))
                        }
                        "category" => {
                            return Err(err(
                                "`category` is set by the query wrapper (`SortingAlgorithm<X>`, \
                                 `Selection<X>`, `RotationAlg<_>`, ...), not by the component; \
                                 delete this line",
                            ))
                        }
                        "menu" => {
                            c.menu = rest
                                .split('/')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect()
                        }
                        "adaptive" => {
                            c.adaptive = rest.trim() == "true";
                        }
                        "nondeterministic" => {
                            c.nondeterministic = rest.trim() == "true";
                        }
                        "max_input" => {
                            c.max_input = Some(
                                rest.trim()
                                    .parse()
                                    .map_err(|_| err("max_input needs an integer"))?,
                            );
                        }
                        "slot" => {
                            let mut it = rest.split_whitespace();
                            let name = it.next().ok_or_else(|| err("slot needs a name"))?;
                            let role = it.next().ok_or_else(|| err("slot needs a role"))?;
                            if let Some(extra) = it.next() {
                                return Err(err(&format!(
                                    "slot takes a name and a role, found `{extra}` after them"
                                )));
                            }
                            c.params.push(Param {
                                name: name.to_string(),
                                kind: ParamKind::Type { role: role.to_string() },
                            });
                        }
                        "const" => {
                            // `const <name>` or `const <name> values <v>...`
                            let mut it = rest.split_whitespace();
                            let name = it.next().ok_or_else(|| err("const needs a name"))?;
                            let values = match it.next() {
                                None => Vec::new(),
                                Some("values") => it.map(str::to_string).collect(),
                                Some(other) => {
                                    return Err(err(&format!(
                                        "const takes `values <v>...` after its name, found `{other}`"
                                    )))
                                }
                            };
                            c.params.push(Param {
                                name: name.to_string(),
                                kind: ParamKind::Const { values },
                            });
                        }
                        other => return Err(err(&format!("unknown keyword `{other}`"))),
                    }
                }
            }
        }
        if let Some(c) = cur.take() {
            reg.insert_component(c).map_err(|m| format!("registry end: {m}"))?;
        }
        Ok(reg)
    }

    pub fn get(&self, name: &str) -> Option<&Component> {
        self.components.get(name)
    }

    /// Iterate every component in PARSE ORDER. Used by the AVBS frontend to index
    /// type-heads (the leading identifier of each `type` template) → component
    /// name; parse order makes that index deterministic and first-wins resolve to
    /// the earliest-declared component on a head collision.
    pub fn iter(&self) -> impl Iterator<Item = &Component> {
        self.order.iter().filter_map(move |n| self.components.get(n))
    }

    /// The component that declares `head` as its type head, the first declared
    /// when several share it (they share one import path).
    pub fn component_with_head(&self, head: &str) -> Option<&Component> {
        self.components.get(self.heads.get(head)?)
    }

    /// All components that can satisfy `role`, base or derived, name-sorted for
    /// deterministic enumeration order.
    pub fn providing(&self, role: &str) -> Vec<&Component> {
        match self.derived.get(role) {
            Some(def) => def.members.iter().filter_map(|name| self.components.get(name)).collect(),
            None => {
                let mut v: Vec<&Component> = self
                    .components
                    .values()
                    .filter(|c| c.provides.iter().any(|r| r == role))
                    .collect();
                v.sort_by(|a, b| a.name.cmp(&b.name));
                v
            }
        }
    }

    /// Whether `comp` is one of the components `providing(role)` returns.
    pub fn has_role(&self, comp: &Component, role: &str) -> bool {
        match self.derived.get(role) {
            Some(def) => def.members.binary_search(&comp.name).is_ok(),
            None => comp.provides.iter().any(|r| r == role),
        }
    }

    /// Every role `comp` joins, base roles in `provides` order then the derived
    /// roles in name order.
    pub fn roles_of<'a>(&'a self, comp: &'a Component) -> Vec<&'a str> {
        let base = comp.provides.iter().map(String::as_str);
        let derived = self.derived.iter().filter(|(_, def)| def.members.binary_search(&comp.name).is_ok());
        base.chain(derived.map(|(name, _)| name.as_str())).collect()
    }

    /// Every role that has a member, base and derived, name-sorted.
    pub fn roles(&self) -> BTreeSet<&str> {
        let base = self.components.values().flat_map(|c| c.provides.iter().map(String::as_str));
        base.chain(self.derived.keys().map(String::as_str)).collect()
    }

    /// Check every `role` line and fix each derived role's members, memoising a
    /// role once so the roles defined over it read it once. A role that is also
    /// provided, an unknown term, and a definition that reaches itself fail, each
    /// naming the role.
    fn resolve_roles(&mut self) -> Result<(), String> {
        let mut memo = HashMap::new();
        for name in self.derived.keys() {
            self.derived_members(name, &mut Vec::new(), &mut memo)?;
        }
        for (name, members) in memo {
            self.derived.get_mut(&name).expect("memo holds derived roles").members = members.into_iter().collect();
        }
        Ok(())
    }

    /// The members of derived role `role`, with `path` the derived roles being
    /// resolved above it, to catch a definition that reaches itself.
    fn derived_members(
        &self,
        role: &str,
        path: &mut Vec<String>,
        memo: &mut HashMap<String, BTreeSet<String>>,
    ) -> Result<BTreeSet<String>, String> {
        if let Some(done) = memo.get(role) {
            return Ok(done.clone());
        }
        if let Some(at) = path.iter().position(|r| r == role) {
            let cycle = path[at..].iter().map(String::as_str).chain([role]).collect::<Vec<_>>();
            return Err(format!("role `{role}` is defined in terms of itself: {}", cycle.join(" -> ")));
        }
        if let Some(clash) = self.iter().find(|c| c.provides.iter().any(|r| r == role)) {
            return Err(format!(
                "role `{role}` is defined by a `role` line and also provided by `{}`; \
                 a role is base or derived, never both",
                clash.name
            ));
        }
        path.push(role.to_string());
        let mut members = BTreeSet::new();
        for (op, term) in &self.derived[role].terms {
            let term_members = self.term_members(role, term, path, memo)?;
            match op {
                Op::Add => members.extend(term_members),
                Op::Remove => members.retain(|name| !term_members.contains(name)),
            }
        }
        path.pop();
        memo.insert(role.to_string(), members.clone());
        Ok(members)
    }

    /// The members a term of `owner`'s definition stands for: a derived role, a
    /// base role, or one component.
    fn term_members(
        &self,
        owner: &str,
        term: &str,
        path: &mut Vec<String>,
        memo: &mut HashMap<String, BTreeSet<String>>,
    ) -> Result<BTreeSet<String>, String> {
        let provided: BTreeSet<String> =
            self.iter().filter(|c| c.provides.iter().any(|r| r == term)).map(|c| c.name.clone()).collect();
        let derived = self.derived.contains_key(term);
        match (derived || !provided.is_empty(), self.components.contains_key(term)) {
            (true, true) => Err(format!("role `{owner}`: `{term}` is both a role and a component, rename one")),
            (false, false) => Err(format!("role `{owner}`: unknown term `{term}`, neither a role nor a component")),
            (false, true) => Ok(BTreeSet::from([term.to_string()])),
            (true, false) if derived => self.derived_members(term, path, memo),
            (true, false) => Ok(provided),
        }
    }
}

/// A role or component name: the characters of a Rust identifier.
fn is_name(word: &str) -> bool {
    !word.is_empty() && word.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The text after `role`: `<Name> = <term> (('|' | '-') <term>)*`.
fn parse_role(text: &str, krate: Option<&str>) -> Result<(String, DerivedRole), String> {
    let (name, expr) = text.split_once('=').ok_or("`role` takes `<Name> = <term> ...`")?;
    let name = name.trim();
    if !is_name(name) {
        return Err(format!("`{name}` is not a role name"));
    }
    let fail = |m: String| format!("role `{name}`: {m}");
    let spaced = expr.replace('|', " | ").replace('-', " - ");
    let mut terms: Vec<(Op, String)> = Vec::new();
    let mut op = Op::Add;
    let mut want_term = true;
    for token in spaced.split_whitespace() {
        match (want_term, token) {
            (_, word) if !matches!(word, "|" | "-") && !is_name(word) => {
                return Err(fail(format!("`{word}` is not a role or component name")))
            }
            (true, "|" | "-") => {
                return Err(fail(format!("expected a role or component name, found `{token}`")))
            }
            (true, word) => {
                terms.push((op, word.to_string()));
                want_term = false;
            }
            (false, "|" | "-") => {
                op = if token == "|" { Op::Add } else { Op::Remove };
                want_term = true;
            }
            (false, word) => return Err(fail(format!("expected `|` or `-` before `{word}`"))),
        }
    }
    match (terms.is_empty(), want_term) {
        (true, _) => Err(fail("needs a term after `=`".to_string())),
        (false, true) => Err(fail("the definition ends with an operator".to_string())),
        (false, false) => {
            let role = DerivedRole { krate: krate.map(str::to_string), terms, members: Vec::new() };
            Ok((name.to_string(), role))
        }
    }
}

fn split_first_word(line: &str) -> (&str, &str) {
    match line.find(char::is_whitespace) {
        Some(i) => (&line[..i], &line[i + 1..]),
        None => (line, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_body(body: &str) -> Result<Registry, String> {
        Registry::parse(&format!("component c\n  type T\n{body}\nend\n"))
    }

    fn const_values(reg: &Registry, name: &str) -> Vec<String> {
        match &reg.get("c").unwrap().param(name).unwrap().kind {
            ParamKind::Const { values } => values.clone(),
            other => panic!("`{name}` should be a const, got {other:?}"),
        }
    }

    #[test]
    fn slot_is_a_name_and_a_role() {
        let reg = parse_body("  slot x Role").unwrap();
        assert_eq!(reg.get("c").unwrap().param_role("x"), Some("Role"));
        let err = parse_body("  slot x Role filler").unwrap_err();
        assert!(err.contains("slot takes a name and a role, found `filler` after them"), "got: {err}");
    }

    #[test]
    fn const_is_a_name_with_an_optional_value_set() {
        let reg = parse_body("  const lit\n  const flag values false true").unwrap();
        assert!(const_values(&reg, "lit").is_empty());
        assert_eq!(const_values(&reg, "flag"), ["false", "true"]);
        for bare_default in ["  const n 16", "  const n 16 values 16 32", "  const flag false"] {
            let err = parse_body(bare_default).unwrap_err();
            assert!(err.contains("const takes `values <v>...` after its name"), "`{bare_default}` got: {err}");
        }
    }

    #[test]
    fn project_is_an_unknown_keyword() {
        let err = parse_body("  project pivot PivotSingle").unwrap_err();
        assert!(err.contains("unknown keyword `project`"), "got: {err}");
    }

    #[test]
    fn components_sharing_a_type_head_must_import_it_from_one_path() {
        let pair = |path_b: &str| {
            format!("component a\n  type T\n  uses x::T\nend\ncomponent b\n  type T<{{s}}>\n  uses {path_b}\n  slot s R\nend\n")
        };
        assert!(Registry::parse(&pair("x::T")).is_ok());
        let err = Registry::parse(&pair("y::T")).unwrap_err();
        assert!(err.contains("`a` and `b` share the type head `T`"), "got: {err}");
        assert!(err.contains("`x::T` and `y::T`"), "got: {err}");
    }

    #[test]
    fn a_fragment_records_its_owning_crate_and_a_plain_parse_records_none() {
        let text = "component c\n  type C\nend\n";
        let owned = Registry::parse_fragment(text, "k").unwrap();
        assert_eq!(owned.get("c").unwrap().krate.as_deref(), Some("k"));
        assert_eq!(Registry::parse(text).unwrap().get("c").unwrap().krate, None);
    }

    #[test]
    fn merged_fragments_share_one_namespace_that_names_both_crates_on_a_clash() {
        let err = Registry::from_fragments([
            ("k1", "component c\n  type A\nend\n"),
            ("k2", "component c\n  type B\nend\n"),
        ])
        .unwrap_err();
        assert!(err.contains("duplicate component `c`"), "got: {err}");
        assert!(err.contains("`k1` and `k2`"), "got: {err}");
    }

    #[test]
    fn one_head_in_two_crates_is_two_import_paths() {
        let err = Registry::from_fragments([
            ("k1", "component a\n  type T\nend\n"),
            ("k2", "component b\n  type T\nend\n"),
        ])
        .unwrap_err();
        assert!(err.contains("`k1::T` and `k2::T`"), "got: {err}");
        let one_crate = "component a\n  type T\nend\ncomponent b\n  type T<{s}>\n  slot s R\nend\n";
        assert!(Registry::parse_fragment(one_crate, "k1").is_ok());
    }

    // One component per `(name, provides line)`, each with a type of its own.
    fn catalog(tags: &[(&str, &str)]) -> String {
        let one = |(name, provides): &(&str, &str)| {
            format!("component {name}\n  type T{name}\n  provides {provides}\nend\n")
        };
        tags.iter().map(one).collect()
    }

    fn pivots() -> String {
        catalog(&[("first", "Pivot"), ("middle", "Pivot"), ("ninther", "Pivot"), ("last", "Pivot")])
    }

    fn members(reg: &Registry, role: &str) -> Vec<String> {
        reg.providing(role).iter().map(|c| c.name.clone()).collect()
    }

    #[test]
    fn a_role_line_defines_a_derived_role_over_a_base_one() {
        let reg = Registry::parse(&format!("{}role Heap = Pivot - last\n", pivots())).unwrap();
        assert_eq!(members(&reg, "Heap"), ["first", "middle", "ninther"]);
        assert_eq!(members(&reg, "Pivot"), ["first", "last", "middle", "ninther"]);
    }

    #[test]
    fn a_derived_role_equals_the_base_role_spelled_member_by_member() {
        let by_hand = catalog(&[
            ("first", "Pivot Heap Beap"),
            ("middle", "Pivot Heap Beap"),
            ("ninther", "Pivot Heap"),
            ("last", "Pivot"),
        ]);
        let by_rule = format!("{}role Heap = Pivot - last\nrole Beap = Heap - ninther\n", pivots());
        let (by_hand, by_rule) = (Registry::parse(&by_hand).unwrap(), Registry::parse(&by_rule).unwrap());
        for role in ["Pivot", "Heap", "Beap"] {
            assert_eq!(members(&by_rule, role), members(&by_hand, role), "role `{role}`");
        }
    }

    #[test]
    fn terms_apply_left_to_right_and_a_term_may_be_a_component() {
        let text = pivots() + &catalog(&[("odd", "Other")]);
        let sets = |line: &str| members(&Registry::parse(&format!("{text}{line}\n")).unwrap(), "R");
        assert_eq!(sets("role R = Pivot - last - ninther"), ["first", "middle"]);
        assert_eq!(sets("role R = Pivot - last | last"), ["first", "last", "middle", "ninther"]);
        assert_eq!(sets("role R = Pivot - Other | odd"), ["first", "last", "middle", "ninther", "odd"]);
        assert_eq!(sets("role R = odd | middle"), ["middle", "odd"]);
        assert_eq!(sets("role R = Other"), ["odd"]);
        assert_eq!(sets("role R = Pivot - Pivot"), Vec::<String>::new());
    }

    #[test]
    fn a_derived_role_may_be_defined_over_a_derived_role_in_any_order() {
        let rules = "role Beap = Heap - ninther\nrole Heap = Pivot - last\n";
        let reg = Registry::parse(&format!("{}{rules}", pivots())).unwrap();
        assert_eq!(members(&reg, "Beap"), ["first", "middle"]);
    }

    #[test]
    fn a_role_both_provided_and_derived_is_an_error_naming_the_role() {
        let text = format!("{}role Pivot = last\n", pivots());
        let err = Registry::parse(&text).unwrap_err();
        assert!(err.contains("`Pivot`"), "got: {err}");
        assert!(err.contains("base or derived, never both"), "got: {err}");
    }

    #[test]
    fn an_unknown_term_is_an_error_naming_the_role_and_the_term() {
        let err = Registry::parse(&format!("{}role Heap = Pivot - lost\n", pivots())).unwrap_err();
        assert!(err.contains("`Heap`") && err.contains("unknown term `lost`"), "got: {err}");
    }

    #[test]
    fn a_term_naming_a_role_and_a_component_is_ambiguous() {
        let text = catalog(&[("same", "Pivot"), ("Pivot", "Other")]);
        let err = Registry::parse(&format!("{text}role R = Pivot\n")).unwrap_err();
        assert!(err.contains("`R`") && err.contains("`Pivot` is both a role and a component"), "got: {err}");
    }

    #[test]
    fn a_definition_that_reaches_itself_is_an_error_naming_the_role() {
        for rules in ["role Loop = Loop | first\n", "role Loop = Other\nrole Other = Loop - first\n"] {
            let err = Registry::parse(&format!("{}{rules}", pivots())).unwrap_err();
            assert!(err.contains("role `Loop`") && err.contains("defined in terms of itself"), "`{rules}` got: {err}");
        }
    }

    #[test]
    fn a_malformed_role_line_is_an_error_naming_the_role() {
        let bad = [
            ("role Heap Pivot - last", "takes `<Name> = <term>"),
            ("role Heap =", "needs a term"),
            ("role Heap = Pivot -", "ends with an operator"),
            ("role Heap = - last", "expected a role or component name, found `-`"),
            ("role Heap = Pivot last", "expected `|` or `-` before `last`"),
            ("role Heap = Pivot & last", "`&` is not a role or component name"),
        ];
        for (line, expected) in bad {
            let err = Registry::parse(&format!("{}{line}\n", pivots())).unwrap_err();
            assert!(err.contains(expected), "`{line}` got: {err}");
        }
    }

    #[test]
    fn a_role_line_inside_a_component_is_an_error() {
        let err = Registry::parse("component c\n  type C\n  role R = c\nend\n").unwrap_err();
        assert!(err.contains("`role` inside component `c`"), "got: {err}");
    }

    #[test]
    fn two_definitions_of_one_derived_role_are_an_error_that_names_both_crates() {
        let base = ("k0", pivots());
        let one = |krate| (krate, "role Heap = Pivot - last\n".to_string());
        let err = Registry::from_fragments(
            [base.clone(), one("k1"), one("k2")].iter().map(|(krate, text)| (*krate, text.as_str())),
        )
        .unwrap_err();
        assert!(err.contains("duplicate derived role `Heap`") && err.contains("`k1` and `k2`"), "got: {err}");
        let twice = format!("{}role Heap = Pivot - last\nrole Heap = Pivot\n", pivots());
        assert!(Registry::parse(&twice).unwrap_err().contains("duplicate derived role `Heap`"));
    }

    #[test]
    fn a_fragment_may_derive_a_role_from_the_components_of_another() {
        let rule = "role Heap = Pivot - last\n";
        let reg = Registry::from_fragments([("pivots", pivots().as_str()), ("heap", rule)]).unwrap();
        assert_eq!(members(&reg, "Heap"), ["first", "middle", "ninther"]);
        let err = Registry::from_fragments([("heap", rule)]).unwrap_err();
        assert!(err.contains("`Heap`") && err.contains("unknown term `Pivot`"), "got: {err}");
    }

    #[test]
    fn a_category_line_is_an_error_that_names_the_wrapper() {
        let err = Registry::parse("component c\n  type C\n  category Sort\nend\n").unwrap_err();
        assert!(err.contains("registry line 3"), "got: {err}");
        assert!(err.contains("`category`"), "got: {err}");
        assert!(err.contains("SortingAlgorithm"), "got: {err}");
    }
}
