//! Stage 1 — the text catalog: parse `registry.spec` into a component graph.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ParamKind {
    /// Filled by a nested component that `provides` `role`. `default` is the
    /// component name used when the slot is omitted; `None` = required.
    Type { role: String, default: Option<String> },
    /// Filled by a literal token (integer OR `true`/`false` — real const
    /// generics include `const N: usize` and `const PING_PONG: bool`). Matched
    /// either by name (`ping_pong = true`) or positionally (`insertion<32>`).
    /// `values` is the declared finite set of "neat values" a quantified const
    /// hole (`threshold = *`/`?`) ranges over — membership only, never
    /// arithmetic. Empty = the const is literal-only and a hole degenerates to
    /// `default`.
    Const { default: Option<String>, values: Vec<String> },
    /// A REFERENCEABLE STRUCTURAL PARAM that never appears in the type template
    /// and is never emitted. It exists only so a refinement (`Partition[pivot =
    /// p]`) can name it and the solver can thread a role constraint through a
    /// shared variable. A component that does NOT declare a `project pivot`
    /// simply cannot satisfy `Partition[pivot = …]` — that exclusion *is* the
    /// arity filter. rustc remains the redundant backstop on the emitted type.
    Project { role: String },
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
    /// emit backend registers it into `NONDETERMINISTIC_ALGOS` so the
    /// determinism property-check skips it — the contract-as-data analogue of
    /// the hand-written `register_nondeterministic!`.
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
    /// A referenceable param by name — any of Type/Const/Project. A refinement
    /// `[name = …]` is only valid if this returns `Some`.
    pub fn param(&self, name: &str) -> Option<&Param> {
        self.params.iter().find(|p| p.name == name)
    }

    /// The role a referenceable param contributes to whatever is bound to it.
    /// `Type`/`Project` carry a role; `Const` carries none (its constraint is
    /// the value set, not a role).
    pub fn param_role(&self, name: &str) -> Option<&str> {
        match &self.param(name)?.kind {
            ParamKind::Type { role, .. } | ParamKind::Project { role } => Some(role),
            ParamKind::Const { .. } => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct Registry {
    components: HashMap<String, Component>,
}

impl Registry {
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
                        reg.components.insert(c.name.clone(), c);
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
                    reg.components.insert(c.name.clone(), c);
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
                            let default = it.next().map(str::to_string);
                            c.params.push(Param {
                                name: name.to_string(),
                                kind: ParamKind::Type { role: role.to_string(), default },
                            });
                        }
                        "const" => {
                            // `const <name> [<default>] [values <v>...]`
                            let mut it = rest.split_whitespace();
                            let name = it.next().ok_or_else(|| err("const needs a name"))?;
                            let mut default = None;
                            let mut values = Vec::new();
                            match it.next() {
                                Some("values") => values.extend(it.map(str::to_string)),
                                Some(d) => {
                                    default = Some(d.to_string());
                                    if it.next() == Some("values") {
                                        values.extend(it.map(str::to_string));
                                    }
                                }
                                None => {}
                            }
                            c.params.push(Param {
                                name: name.to_string(),
                                kind: ParamKind::Const { default, values },
                            });
                        }
                        "project" => {
                            let mut it = rest.split_whitespace();
                            let name = it.next().ok_or_else(|| err("project needs a name"))?;
                            let role = it.next().ok_or_else(|| err("project needs a role"))?;
                            c.params.push(Param {
                                name: name.to_string(),
                                kind: ParamKind::Project { role: role.to_string() },
                            });
                        }
                        other => return Err(err(&format!("unknown keyword `{other}`"))),
                    }
                }
            }
        }
        if let Some(c) = cur.take() {
            reg.components.insert(c.name.clone(), c);
        }
        Ok(reg)
    }

    pub fn get(&self, name: &str) -> Option<&Component> {
        self.components.get(name)
    }

    /// Iterate every component. Used by the AVBS frontend to index type-heads
    /// (the leading identifier of each `type` template) → component name.
    pub fn iter(&self) -> impl Iterator<Item = &Component> {
        self.components.values()
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
