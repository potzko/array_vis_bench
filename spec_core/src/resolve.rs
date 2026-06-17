//! Stage 3 — resolve a [`SpecNode`] against the [`Registry`]: fill defaults,
//! check slot roles/arity, collect imports, and produce a concrete Rust type
//! expression + label.

use std::collections::HashMap;

use crate::registry::{ParamKind, Registry};
use crate::spec::{Arg, SpecNode};

/// One faceted navigation axis: a slot's `role`, the chosen filler's `value`
/// (its HEAD label — `combined`, not `combined<first,mid>`), and a unique
/// `/`-separated `path` encoding nesting. A composite filler contributes its own
/// node plus its sub-slots' axes with the parent slot prefixed onto their paths
/// (`pivot`, then `pivot/a`, `pivot/b`), so the picker navigates a nested type
/// as its own levels.
#[derive(Debug, Clone, PartialEq)]
pub struct AxisNode {
    pub role: String,
    pub value: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Resolved {
    pub type_expr: String,
    pub label: String,
    /// Unioned module paths this concrete type needs in scope.
    pub uses: Vec<String>,
    /// STRUCTURAL faceted axes, pre-order: each direct type slot contributes a
    /// node `(role, head label, slot path)`, immediately followed by the chosen
    /// filler's own axes with their paths prefixed by `"<slot>/"`. A leaf filler
    /// adds just its node; a composite (`combined<a,b>`) adds the node + its
    /// sub-slots. The picker keys navigation on `path`, so a composite becomes
    /// its own level with sub-levels.
    pub axes: Vec<AxisNode>,
    /// `label` with each direct type slot replaced by a role-tagged `{Role}`
    /// hole (consts substituted exactly as in `label`). One hole per axis, in
    /// the label's own visual order. A navigation backend can register this so
    /// the picker fills the holes by role to render the partial type in the
    /// catalog's label syntax. Holes are tagged by role (which the picker
    /// already requires to be unique per component), so it is robust to the
    /// label ordering its slots differently from their declaration order.
    pub label_template: String,
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
    // The label with each direct type slot rewritten to a `{Role}` hole; consts
    // are substituted just like in `label`. Built structurally here (never by
    // matching values back out of the finished label) so a value that contains
    // commas or angle brackets — e.g. `combined<first,mid>` — can't corrupt it.
    let mut label_template = comp.label_tmpl.clone();
    let mut uses = comp.uses.clone();
    let mut axes: Vec<AxisNode> = Vec::new();
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
                // The template keeps this slot as a `{Role}` hole; the picker
                // fills it per axis. (The role is unique per component — the
                // same key the axes carry.)
                label_template = label_template.replace(&hole, &format!("{{{role}}}"));
                // STRUCTURAL axis: this slot's node (its role + the filler's HEAD
                // label, keyed by the slot name), then the filler's own axes with
                // their paths prefixed by this slot — so a composite filler
                // (`combined<a,b>`) navigates as its own level with sub-levels.
                axes.push(AxisNode {
                    role: role.clone(),
                    value: head_label(child_comp, &r.label),
                    path: p.name.clone(),
                });
                for sub in r.axes {
                    axes.push(AxisNode {
                        role: sub.role,
                        value: sub.value,
                        path: format!("{}/{}", p.name, sub.path),
                    });
                }
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
                label_template = label_template.replace(&hole, &value);
            }
            // Structural-only: never appears in the type template, nothing to
            // emit. Its role constraint is enforced by the solver, not here.
            ParamKind::Project { .. } => {}
        }
    }
    Ok(Resolved { type_expr, label, uses, axes, label_template })
}

/// The navigation HEAD label for a filler: a leaf (no type slots) uses its full
/// resolved label (`first`, `insertion: 32`); a composite uses its label template
/// up to the first type-slot hole, with a trailing bracket/space trimmed
/// (`combined<{a}, {b}>` → `combined`). The head is constant across every nested
/// choice, so all `combined<…>` variants group under one `combined` level.
fn head_label(comp: &crate::registry::Component, resolved_label: &str) -> String {
    let first_type_hole = comp
        .params
        .iter()
        .filter(|p| matches!(p.kind, ParamKind::Type { .. }))
        .filter_map(|p| comp.label_tmpl.find(&format!("{{{}}}", p.name)))
        .min();
    match first_type_hole {
        Some(i) => comp.label_tmpl[..i]
            .trim_end_matches(|c: char| c == '<' || c == '(' || c == '[' || c.is_whitespace())
            .to_string(),
        // Leaf (no type slots), or the slots aren't shown in the label → the full
        // resolved label is already the right node value.
        None => resolved_label.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::Registry;
    use crate::spec::parse_spec;

    const REG: &str = include_str!("../registry.spec");

    fn axes_of(spec: &str) -> Vec<(String, String, String)> {
        let reg = Registry::parse(REG).unwrap();
        let (_, node) = parse_spec(spec).unwrap();
        resolve(&node, &reg)
            .unwrap()
            .axes
            .into_iter()
            .map(|a| (a.role, a.value, a.path))
            .collect()
    }

    #[test]
    fn nested_filler_becomes_child_axes_with_prefixed_paths() {
        // A quick_sort whose pivot is the composite `combined<first, middle>`:
        // the pivot node carries the HEAD `combined`, then its a/b sub-slots
        // appear as `pivot/a`, `pivot/b` — the structural nesting.
        let axes = axes_of(
            "quick_sort< partition = LL_partition, \
                         pivot = combined< a = first_element, b = middle_element >, \
                         small_sort = no_small_sort >",
        );
        assert_eq!(
            axes,
            vec![
                ("Partition".into(), "LL".into(), "partition".into()),
                ("Pivot".into(), "combined".into(), "pivot".into()),
                ("PivotSingle".into(), "first".into(), "pivot/a".into()),
                ("PivotSingle".into(), "mid".into(), "pivot/b".into()),
                ("SmallSort".into(), "none".into(), "small_sort".into()),
            ]
        );
    }

    #[test]
    fn leaf_filler_is_a_single_flat_axis() {
        // A single (non-composite) pivot → one `pivot` node, no children.
        let axes = axes_of(
            "quick_sort< partition = LL_partition, pivot = first_element, small_sort = no_small_sort >",
        );
        assert_eq!(
            axes,
            vec![
                ("Partition".into(), "LL".into(), "partition".into()),
                ("Pivot".into(), "first".into(), "pivot".into()),
                ("SmallSort".into(), "none".into(), "small_sort".into()),
            ]
        );
    }

    #[test]
    fn head_label_trims_to_the_component_head() {
        let reg = Registry::parse(REG).unwrap();
        // composite → head only
        assert_eq!(head_label(reg.get("combined").unwrap(), "combined<first, mid>"), "combined");
        // leaf → full resolved label
        assert_eq!(head_label(reg.get("first_element").unwrap(), "first"), "first");
    }
}
