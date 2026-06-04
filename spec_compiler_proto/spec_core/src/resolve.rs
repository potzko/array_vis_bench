//! Stage 3 — resolve a [`SpecNode`] against the [`Registry`]: fill defaults,
//! check slot roles/arity, collect imports, and produce a concrete Rust type
//! expression + label.

use std::collections::HashMap;

use crate::registry::{ParamKind, Registry};
use crate::spec::{Arg, SpecNode};

#[derive(Debug, Clone)]
pub struct Resolved {
    pub type_expr: String,
    pub label: String,
    /// Unioned module paths this concrete type needs in scope.
    pub uses: Vec<String>,
}

pub fn resolve(node: &SpecNode, reg: &Registry) -> Result<Resolved, String> {
    let comp = reg
        .get(&node.name)
        .ok_or_else(|| format!("unknown component `{}`", node.name))?;

    // Split provided args into named slots, named consts, positional consts.
    let mut named_slots: HashMap<String, &SpecNode> = HashMap::new();
    let mut named_consts: HashMap<String, String> = HashMap::new();
    let mut pos_consts: Vec<String> = Vec::new();
    for arg in &node.args {
        match arg {
            Arg::Slot { name, value } => {
                let known = comp
                    .params
                    .iter()
                    .any(|p| &p.name == name && matches!(p.kind, ParamKind::Type { .. }));
                if !known {
                    return Err(format!("`{}` has no slot named `{name}`", node.name));
                }
                named_slots.insert(name.clone(), value);
            }
            Arg::NamedConst { name, value } => {
                let known = comp
                    .params
                    .iter()
                    .any(|p| &p.name == name && matches!(p.kind, ParamKind::Const { .. }));
                if !known {
                    return Err(format!("`{}` has no const named `{name}`", node.name));
                }
                named_consts.insert(name.clone(), value.clone());
            }
            Arg::Const(v) => pos_consts.push(v.clone()),
        }
    }

    let mut type_expr = comp.type_tmpl.clone();
    let mut label = comp.label_tmpl.clone();
    let mut uses = comp.uses.clone();
    let mut pos_iter = pos_consts.into_iter();

    for p in &comp.params {
        let hole = format!("{{{}}}", p.name);
        match &p.kind {
            ParamKind::Type { role, default } => {
                let child = match named_slots.get(&p.name) {
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
                // ROLE CHECK. NOTE: this validates each slot's filler against
                // that slot's role *independently*. It CANNOT express a
                // cross-slot constraint (e.g. "partition arity must match pivot
                // arity") — see findings. rustc remains the backstop there.
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
                for u in r.uses {
                    if !uses.contains(&u) {
                        uses.push(u);
                    }
                }
            }
            ParamKind::Const { default, .. } => {
                let value = named_consts
                    .get(&p.name)
                    .cloned()
                    .or_else(|| pos_iter.next())
                    .or_else(|| default.clone())
                    .ok_or_else(|| {
                        format!("`{}` needs a value for const `{}`", node.name, p.name)
                    })?;
                type_expr = type_expr.replace(&hole, &value);
                label = label.replace(&hole, &value);
            }
            // Structural-only: never appears in the type template, nothing to
            // emit. Its role constraint is enforced by the solver, not here.
            ParamKind::Project { .. } => {}
        }
    }
    Ok(Resolved { type_expr, label, uses })
}
