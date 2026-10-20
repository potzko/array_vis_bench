//! Stage 1 — the text catalog: parse `registry.spec` into a component graph.

use std::collections::HashMap;

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
    pub provides: Vec<String>,
    pub params: Vec<Param>,
    /// Module paths to bring into scope for `type_tmpl` to resolve, e.g.
    /// `quick_sort_lib::quick_sort::QuickSort`. Mirrors the real `uses = [...]`
    /// family field. Unioned with nested children's `uses` during resolve.
    pub uses: Vec<String>,
    /// Which `AlgorithmEntry` driver/battery this component emits as. `None`
    /// for non-emittable components (slot fillers like pivots/partitions). The
    /// `emit_entries` backend reads this on a query's ROOT component.
    pub category: Option<String>,
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

impl Component {
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
}

impl Registry {
    /// Insert a component, recording its parse position for ordered iteration.
    fn insert_component(&mut self, c: Component) {
        if !self.components.contains_key(&c.name) {
            self.order.push(c.name.clone());
        }
        self.components.insert(c.name.clone(), c);
    }

    pub fn parse(text: &str) -> Result<Registry, String> {
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
                "component" => {
                    if let Some(c) = cur.take() {
                        reg.insert_component(c);
                    }
                    cur = Some(Component {
                        name: rest.trim().to_string(),
                        type_tmpl: String::new(),
                        label_tmpl: String::new(),
                        provides: vec![],
                        params: vec![],
                        uses: vec![],
                        category: None,
                        adaptive: false,
                        nondeterministic: false,
                        max_input: None,
                        menu: vec![],
                    });
                }
                "end" => {
                    let c = cur.take().ok_or_else(|| err("`end` outside a component"))?;
                    reg.insert_component(c);
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
                        "category" => c.category = Some(rest.trim().to_string()),
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
            reg.insert_component(c);
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

    /// All components that can satisfy `role`, name-sorted for deterministic
    /// enumeration order.
    pub fn providing(&self, role: &str) -> Vec<&Component> {
        let mut v: Vec<&Component> = self
            .components
            .values()
            .filter(|c| c.provides.iter().any(|r| r == role))
            .collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
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
}
