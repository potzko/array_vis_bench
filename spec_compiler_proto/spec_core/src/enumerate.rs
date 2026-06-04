//! Stage 0 (mode 2 only) — a program produces the spec trees. Bounded by
//! literal nesting depth; role filtering guarantees only legal trees are built.

use crate::registry::{Component, ParamKind, Registry};
use crate::spec::{Arg, SpecNode};

/// Produce every legal spec tree rooted at `root_role`, bounded by literal
/// nesting `max_depth`. Termination is by depth — an honest, intuitive knob —
/// and role filtering guarantees no illegal combo is ever produced (so no
/// emitted code fails to compile). Const slots take their registry default.
pub fn enumerate(reg: &Registry, root_role: &str, max_depth: usize) -> Vec<SpecNode> {
    let mut out = Vec::new();
    for comp in reg.providing(root_role) {
        out.extend(enumerate_component(reg, comp, max_depth));
    }
    out
}

fn enumerate_component(reg: &Registry, comp: &Component, depth: usize) -> Vec<SpecNode> {
    let type_slots: Vec<_> = comp
        .params
        .iter()
        .filter(|p| matches!(p.kind, ParamKind::Type { .. }))
        .collect();

    if type_slots.is_empty() {
        // Leaf (or const-only): one variant, consts default at resolve time.
        return vec![SpecNode { name: comp.name.clone(), args: vec![] }];
    }
    if depth == 0 {
        return vec![]; // can't nest further on this path
    }

    // Per-slot option lists, then cartesian product.
    let mut slot_opts: Vec<(String, Vec<SpecNode>)> = Vec::new();
    for p in &type_slots {
        if let ParamKind::Type { role, .. } = &p.kind {
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
                .map(|(name, value)| Arg::Named { name, value })
                .collect(),
        })
        .collect()
}
