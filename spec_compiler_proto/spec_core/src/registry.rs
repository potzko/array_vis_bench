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
    Const { default: Option<String> },
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
                            let mut it = rest.split_whitespace();
                            let name = it.next().ok_or_else(|| err("const needs a name"))?;
                            let default = it.next().map(str::to_string);
                            c.params.push(Param {
                                name: name.to_string(),
                                kind: ParamKind::Const { default },
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
