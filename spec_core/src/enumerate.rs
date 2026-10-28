//! Stage 0 (mode 2 only) — a program produces the spec trees. Bounded by
//! literal nesting depth; role filtering guarantees only legal trees are built.

use std::collections::HashSet;

use crate::registry::{Component, ParamKind, Registry};
use crate::spec::{Arg, SpecNode};

/// Produce every legal spec tree rooted at `root_role`, bounded by literal
/// nesting `max_depth`. Termination is by depth — an honest, intuitive knob —
/// and role filtering guarantees no illegal combo is ever produced (so no
/// emitted code fails to compile). Const slots stay unfilled: a component that
/// has one resolves only once a query supplies it.
pub fn enumerate(reg: &Registry, root_role: &str, max_depth: usize) -> Vec<SpecNode> {
    let mut out = Vec::new();
    for comp in reg.providing(root_role) {
        out.extend(enumerate_component(reg, comp, max_depth));
    }
    out
}

/// Like [`enumerate`], but with an OCCURS CHECK: no component may appear inside
/// its own subtree. This makes a RECURSIVE role's population FINITE — only
/// infinite self-nesting is removed; every acyclic tree (including bounded use of
/// a recursive component) is kept. `finite(_)` uses this so a cyclic role graph
/// enumerates to a finite set instead of relying on the `depth` cap + tripping the
/// cardinality guard. `depth` still bounds non-recursive nesting.
pub fn enumerate_finite(reg: &Registry, root_role: &str, max_depth: usize) -> Vec<SpecNode> {
    let mut out = Vec::new();
    let mut path: Vec<String> = Vec::new();
    for comp in reg.providing(root_role) {
        out.extend(enumerate_component_occurs(reg, comp, max_depth, &mut path));
    }
    out
}

/// Like [`enumerate`], but AFFINE: each component name appears at most once in a
/// composition, across siblings as well as along a path. The population of a
/// recursive role is finite because every nesting spends a name, so no cycle
/// analysis is needed. The used set belongs to the subtree rooted at `root_role`,
/// so it starts empty for each provider of it.
pub fn enumerate_affine(reg: &Registry, root_role: &str, max_depth: usize) -> Vec<SpecNode> {
    let mut out = Vec::new();
    let mut used: HashSet<String> = HashSet::new();
    for comp in reg.providing(root_role) {
        walk_affine(reg, comp, max_depth, &mut used, &mut |node, _| out.push(node));
    }
    out
}

/// What to do with each finished subtree. It runs while the subtree's names are
/// still in the used set, so the caller can go on filling its remaining slots
/// against them; the names are popped when the walk backtracks.
type Emit<'a> = &'a mut dyn FnMut(SpecNode, &mut HashSet<String>);

fn walk_affine(
    reg: &Registry,
    comp: &Component,
    depth: usize,
    used: &mut HashSet<String>,
    emit: Emit,
) {
    if !used.insert(comp.name.clone()) {
        return;
    }
    let slots: Vec<(&str, &str)> = comp
        .params
        .iter()
        .filter_map(|p| match &p.kind {
            ParamKind::Type { role } => Some((p.name.as_str(), role.as_str())),
            ParamKind::Const { .. } => None,
        })
        .collect();
    if slots.is_empty() {
        emit(SpecNode { name: comp.name.clone(), args: vec![] }, used);
    } else if depth > 0 {
        walk_slots(reg, comp, &slots, depth - 1, used, &mut Vec::new(), emit);
    }
    used.remove(&comp.name);
}

/// Fill `slots` left to right: each slot tries every provider of its role that
/// the used set still allows, then the rest of the slots see what it spent.
fn walk_slots(
    reg: &Registry,
    comp: &Component,
    slots: &[(&str, &str)],
    depth: usize,
    used: &mut HashSet<String>,
    filled: &mut Vec<Arg>,
    emit: Emit,
) {
    let Some(((slot, role), rest)) = slots.split_first() else {
        emit(SpecNode { name: comp.name.clone(), args: filled.clone() }, used);
        return;
    };
    for child in reg.providing(role) {
        walk_affine(reg, child, depth, used, &mut |value, used| {
            filled.push(Arg::Slot { name: slot.to_string(), value });
            walk_slots(reg, comp, rest, depth, used, filled, emit);
            filled.pop();
        });
    }
}

/// [`enumerate_component`] threading the ancestor-component path; a child whose
/// component name is already on the path is skipped (the occurs check).
fn enumerate_component_occurs(
    reg: &Registry,
    comp: &Component,
    depth: usize,
    path: &mut Vec<String>,
) -> Vec<SpecNode> {
    let type_slots: Vec<_> = comp
        .params
        .iter()
        .filter(|p| matches!(p.kind, ParamKind::Type { .. }))
        .collect();

    if type_slots.is_empty() {
        return vec![SpecNode { name: comp.name.clone(), args: vec![] }];
    }
    if depth == 0 {
        return vec![];
    }

    path.push(comp.name.clone());
    let mut slot_opts: Vec<(String, Vec<SpecNode>)> = Vec::new();
    for p in &type_slots {
        if let ParamKind::Type { role } = &p.kind {
            let mut opts = Vec::new();
            for child in reg.providing(role) {
                // OCCURS CHECK: never nest a component inside itself.
                if path.iter().any(|n| n == &child.name) {
                    continue;
                }
                opts.extend(enumerate_component_occurs(reg, child, depth - 1, path));
            }
            slot_opts.push((p.name.clone(), opts));
        }
    }
    path.pop();

    // A slot with NO options (every provider was pruned by the occurs check) makes
    // this component unrepresentable on this path — no tree.
    if slot_opts.iter().any(|(_, opts)| opts.is_empty()) {
        return vec![];
    }

    let mut combos: Vec<Vec<(String, SpecNode)>> = vec![vec![]];
    for (pname, opts) in &slot_opts {
        let mut next = Vec::new();
        for combo in &combos {
            for opt in opts {
                let mut c = combo.clone();
                c.push((pname.clone(), opt.clone()));
                next.push(c);
            }
        }
        combos = next;
    }

    combos
        .into_iter()
        .map(|combo| SpecNode {
            name: comp.name.clone(),
            args: combo.into_iter().map(|(name, value)| Arg::Slot { name, value }).collect(),
        })
        .collect()
}

fn enumerate_component(reg: &Registry, comp: &Component, depth: usize) -> Vec<SpecNode> {
    let type_slots: Vec<_> = comp
        .params
        .iter()
        .filter(|p| matches!(p.kind, ParamKind::Type { .. }))
        .collect();

    if type_slots.is_empty() {
        // Leaf (or const-only): one variant, its consts are left to the query.
        return vec![SpecNode { name: comp.name.clone(), args: vec![] }];
    }
    if depth == 0 {
        return vec![]; // can't nest further on this path
    }

    // Per-slot option lists, then cartesian product.
    let mut slot_opts: Vec<(String, Vec<SpecNode>)> = Vec::new();
    for p in &type_slots {
        if let ParamKind::Type { role } = &p.kind {
            let mut opts = Vec::new();
            for child in reg.providing(role) {
                opts.extend(enumerate_component(reg, child, depth - 1));
            }
            slot_opts.push((p.name.clone(), opts));
        }
    }

    let mut combos: Vec<Vec<(String, SpecNode)>> = vec![vec![]];
    for (pname, opts) in &slot_opts {
        let mut next = Vec::new();
        for combo in &combos {
            for opt in opts {
                let mut c = combo.clone();
                c.push((pname.clone(), opt.clone()));
                next.push(c);
            }
        }
        combos = next;
    }

    combos
        .into_iter()
        .map(|combo| SpecNode {
            name: comp.name.clone(),
            args: combo
                .into_iter()
                .map(|(name, value)| Arg::Slot { name, value })
                .collect(),
        })
        .collect()
}
