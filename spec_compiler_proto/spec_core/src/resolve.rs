//! Stage 3 — resolve a [`SpecNode`] against the [`Registry`]: fill defaults,
//! check slot roles/arity, and produce a concrete Rust type expression + label.

use std::collections::HashMap;

use crate::registry::{ParamKind, Registry};
use crate::spec::{Arg, SpecNode};

#[derive(Debug, Clone)]
pub struct Resolved {
    pub type_expr: String,
    pub label: String,
}

pub fn resolve(node: &SpecNode, reg: &Registry) -> Result<Resolved, String> {
    let comp = reg
        .get(&node.name)
        .ok_or_else(|| format!("unknown component `{}`", node.name))?;

    let mut named: HashMap<String, &SpecNode> = HashMap::new();
    let mut consts: Vec<i64> = Vec::new();
    for arg in &node.args {
        match arg {
            Arg::Named { name, value } => {
                let known = comp
                    .params
                    .iter()
                    .any(|p| &p.name == name && matches!(p.kind, ParamKind::Type { .. }));
                if !known {
                    return Err(format!("`{}` has no slot named `{name}`", node.name));
                }
                named.insert(name.clone(), value);
            }
            Arg::Const(n) => consts.push(*n),
        }
    }

    let mut type_expr = comp.type_tmpl.clone();
    let mut label = comp.label_tmpl.clone();
    let mut consts_iter = consts.into_iter();

    for p in &comp.params {
        let hole = format!("{{{}}}", p.name);
        match &p.kind {
            ParamKind::Type { role, default } => {
                let child = match named.get(&p.name) {
                    Some(child) => (*child).clone(),
                    None => match default {
                        Some(d) => SpecNode { name: d.clone(), args: vec![] },
                        None => {
                            return Err(format!(
                                "`{}` requires slot `{}` (role {role})",
                                node.name, p.name
                            ))
                        }
                    },
                };
                // ROLE / ARITY CHECK — the payoff of nesting the pivot under the
                // partition: a single-pivot partition cannot accept a dual selector.
                let child_comp = reg
                    .get(&child.name)
                    .ok_or_else(|| format!("unknown component `{}`", child.name))?;
                if !child_comp.provides.iter().any(|r| r == role) {
                    return Err(format!(
                        "slot `{}` of `{}` needs role `{role}`, but `{}` provides {:?}",
                        p.name, node.name, child.name, child_comp.provides
                    ));
                }
                let r = resolve(&child, reg)?;
                type_expr = type_expr.replace(&hole, &r.type_expr);
                label = label.replace(&hole, &r.label);
            }
            ParamKind::Const { default } => {
                let value = consts_iter
                    .next()
                    .or(*default)
                    .ok_or_else(|| format!("`{}` needs a value for const `{}`", node.name, p.name))?;
                type_expr = type_expr.replace(&hole, &value.to_string());
                label = label.replace(&hole, &value.to_string());
            }
        }
    }
    Ok(Resolved { type_expr, label })
}
