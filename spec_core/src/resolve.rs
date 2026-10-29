//! Stage 3, resolve a [`SpecNode`] against the [`Registry`]: check slot
//! roles/arity, collect imports, and produce a concrete Rust type expression +
//! label.

use std::collections::HashMap;

use crate::registry::{Component, ParamKind, Registry};
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
    let mut uses = own_imports(comp, reg)?;
    let mut axes: Vec<AxisNode> = Vec::new();
    let mut pos_iter = pos_consts.into_iter();

    for p in &comp.params {
        let hole = format!("{{{}}}", p.name);
        match &p.kind {
            ParamKind::Type { role } => {
                let child = named_slots
                    .get(&p.name)
                    .map(|child| (*child).clone())
                    .ok_or_else(|| {
                        format!("`{}` requires slot `{}` (role {role})", node.name, p.name)
                    })?;
                // ROLE CHECK. NOTE: this validates each slot's filler against
                // that slot's role *independently*. It CANNOT express a
                // cross-slot constraint (e.g. "partition arity must match pivot
                // arity") — see findings. rustc remains the backstop there.
                let child_comp = reg
                    .get(&child.name)
                    .ok_or_else(|| format!("unknown component `{}`", child.name))?;
                if !reg.has_role(child_comp, role) {
                    return Err(format!(
                        "slot `{}` of `{}` needs role `{role}`, but `{}` provides {:?}",
                        p.name, node.name, child.name, reg.roles_of(child_comp)
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
                    value: head_label(child_comp, &r.label, &r.axes),
                    path: p.name.clone(),
                });
                // A label that opens with a slot hole lends that slot's axis to the
                // head above, so it is not repeated as a sub axis.
                let lent = lent_slot(child_comp);
                for sub in r.axes.into_iter().filter(|sub| Some(sub.path.as_str()) != lent) {
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
            ParamKind::Const { .. } => {
                let value = named_consts
                    .get(&p.name)
                    .cloned()
                    .or_else(|| pos_iter.next())
                    .ok_or_else(|| {
                        format!("`{}` needs a value for const `{}`", node.name, p.name)
                    })?;
                type_expr = type_expr.replace(&hole, &value);
                label = label.replace(&hole, &value);
                label_template = label_template.replace(&hole, &value);
            }
        }
    }
    Ok(Resolved { type_expr, label, uses, axes, label_template })
}

/// Words in a type template that name no importable type: literals, keywords
/// and the primitive types.
const NOT_IMPORTED: &[&str] = &[
    "true", "false", "dyn", "impl", "mut", "const", "fn", "for", "Self", "bool", "char", "str",
    "usize", "isize", "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32",
    "f64",
];

/// The identifiers of a `type` template outside its `{hole}`s, numbers and
/// [`NOT_IMPORTED`] words dropped.
fn template_idents(type_tmpl: &str) -> impl Iterator<Item = &str> {
    let outside_holes = type_tmpl.split('{').enumerate().map(|(i, text)| match i {
        0 => text,
        _ => text.split_once('}').map_or("", |(_, rest)| rest),
    });
    outside_holes
        .flat_map(|text| text.split(|c: char| !(c.is_alphanumeric() || c == '_')))
        .filter(|word| !word.is_empty() && !word.starts_with(|c: char| c.is_ascii_digit()))
        .filter(|word| !NOT_IMPORTED.contains(word))
}

/// The imports a component's own `type` template needs, not counting its fillers'.
/// An identifier that heads a component is imported from that component's path,
/// and any other one, the template's own head and a baked helper such as `AryPair`
/// among them, from the owning crate. A component that lists `uses` gets exactly
/// those instead, the override for a type the owning crate cannot reach.
fn own_imports(comp: &Component, reg: &Registry) -> Result<Vec<String>, String> {
    if !comp.uses.is_empty() {
        return Ok(comp.uses.clone());
    }
    let krate = comp.krate.as_deref().ok_or_else(|| {
        format!("component `{}` has no owning crate and no `uses`, so no imports", comp.name)
    })?;
    let mut paths: Vec<String> = Vec::new();
    for ident in template_idents(&comp.type_tmpl) {
        let path = reg
            .component_with_head(ident)
            .and_then(Component::head_path)
            .unwrap_or_else(|| format!("{krate}::{ident}"));
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    Ok(paths)
}

/// The first type slot whose hole appears in the label template, with the
/// template text before that hole up to the outermost bracket still open
/// there, minus trailing colons and spaces: `combined<{a}, {b}>` and
/// `stack partition<part: {hp}>` head as `combined` and `stack partition`, so
/// a slot's own name inside the brackets is not part of the head.
fn label_opening(comp: &Component) -> Option<(&str, &str)> {
    let (at, slot) = comp
        .params
        .iter()
        .filter(|p| matches!(p.kind, ParamKind::Type { .. }))
        .filter_map(|p| comp.label_tmpl.find(&format!("{{{}}}", p.name)).map(|i| (i, p.name.as_str())))
        .min()?;
    let before = &comp.label_tmpl[..at];
    let open = before.char_indices().fold(Vec::new(), |mut open, (i, c)| {
        match c {
            '<' | '(' | '[' => open.push(i),
            '>' | ')' | ']' => { open.pop(); }
            _ => {}
        }
        open
    });
    let head = before[..open.first().copied().unwrap_or(before.len())]
        .trim_end_matches(|c: char| c == ':' || c.is_whitespace());
    Some((slot, head))
}

/// The slot a composite is named by when its label template opens with that
/// slot's hole (`{scheme}<{pivot}>`), so the template text before it is empty.
fn lent_slot(comp: &Component) -> Option<&str> {
    label_opening(comp).filter(|(_, head)| head.is_empty()).map(|(slot, _)| slot)
}

/// The navigation HEAD label for a filler: a leaf (no type slots) uses its full
/// resolved label (`first`, `insertion: 32`); a composite uses its label template
/// up to the first type-slot hole, with a trailing bracket/space trimmed
/// (`combined<{a}, {b}>` -> `combined`). The head is constant across every nested
/// choice, so all `combined<...>` variants group under one `combined` level. A
/// template that opens with a hole has no text to use, so the head is the head of
/// that slot's filler, read from the filler's own `axes`.
fn head_label(comp: &Component, resolved_label: &str, axes: &[AxisNode]) -> String {
    match label_opening(comp) {
        Some((slot, "")) => axes
            .iter()
            .find(|a| a.path == slot)
            .map(|a| a.value.clone())
            .expect("every type slot of a resolved composite has an axis"),
        Some((_, head)) => head.to_string(),
        // Leaf (no type slots), or the slots aren't shown in the label -> the full
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
        // A quick_sort whose partition is the dual composite over the composite
        // pivot `combined<first, middle>`: each node carries its HEAD label, and
        // the paths accumulate down the nesting (`partition/pivot/a`).
        let axes = axes_of(
            "quick_sort< partition = pk_dual< pivot = combined< a = first_element, b = middle_element > >, \
                         small_sort = no_small_sort >",
        );
        assert_eq!(
            axes,
            vec![
                ("Partition".into(), "dual".into(), "partition".into()),
                ("PivotDual".into(), "combined".into(), "partition/pivot".into()),
                ("PivotSingle".into(), "first".into(), "partition/pivot/a".into()),
                ("PivotSingle".into(), "mid".into(), "partition/pivot/b".into()),
                ("SmallSort".into(), "none".into(), "small_sort".into()),
            ]
        );
    }

    #[test]
    fn leaf_fillers_are_single_flat_axes() {
        // The single composite's scheme and pivot are leaves: one node each under
        // the partition, no children.
        let axes = axes_of(
            "quick_sort< partition = pk_single< scheme = LL_partition, pivot = first_element >, \
                         small_sort = no_small_sort >",
        );
        assert_eq!(
            axes,
            vec![
                ("Partition".into(), "single".into(), "partition".into()),
                ("PartScheme".into(), "LL".into(), "partition/scheme".into()),
                ("PivotSingle".into(), "first".into(), "partition/pivot".into()),
                ("SmallSort".into(), "none".into(), "small_sort".into()),
            ]
        );
    }

    fn axes_in(fragments: &[(&str, &str)], spec: &str) -> Vec<(String, String, String)> {
        let reg = Registry::from_fragments(fragments.iter().copied()).unwrap();
        let axes = resolve(&parse_spec(spec).unwrap().1, &reg).unwrap().axes;
        axes.into_iter().map(|a| (a.role, a.value, a.path)).collect()
    }

    #[test]
    fn a_slot_named_inside_the_brackets_is_not_part_of_the_head() {
        // `wrap<part: {inner}>` heads as `wrap`: the text before the hole is cut
        // at the bracket still open there, so the slot's own name stays out.
        let fragments = [(
            "lib",
            "component driver\n  type Driver<{f}>\n  label driver<{f}>\n  slot f Fill\nend\n\
             component wrap\n  type Wrap<{inner}>\n  label wrap<part: {inner}>\n  provides Fill\n  slot inner Leaf\nend\n\
             component named\n  type Named<{inner}>\n  label named: {inner}\n  provides Fill\n  slot inner Leaf\nend\n\
             component leaf\n  type Leaf\n  label leaf\n  provides Leaf\nend\n",
        )];
        let axis = |role: &str, value: &str, path: &str| (role.to_string(), value.to_string(), path.to_string());
        assert_eq!(
            axes_in(&fragments, "driver< f = wrap< inner = leaf > >"),
            [axis("Fill", "wrap", "f"), axis("Leaf", "leaf", "f/inner")]
        );
        assert_eq!(
            axes_in(&fragments, "driver< f = named< inner = leaf > >"),
            [axis("Fill", "named", "f"), axis("Leaf", "leaf", "f/inner")]
        );
    }

    fn imports_of(fragments: &[(&str, &str)], spec: &str) -> Result<Vec<String>, String> {
        let reg = Registry::from_fragments(fragments.iter().copied())?;
        let mut uses = resolve(&parse_spec(spec).unwrap().1, &reg)?.uses;
        uses.sort();
        Ok(uses)
    }

    #[test]
    fn a_component_without_uses_imports_its_head_from_its_crate() {
        let leaf = [("leaf_lib", "component leaf\n  type Leaf\nend\n")];
        assert_eq!(imports_of(&leaf, "leaf").unwrap(), ["leaf_lib::Leaf"]);
    }

    #[test]
    fn a_nested_filler_adds_the_crate_that_declares_it() {
        let fragments = [
            ("driver_lib", "component driver\n  type Driver<{f}>\n  slot f Fill\nend\n"),
            ("fill_lib", "component fill\n  type Fill\n  provides Fill\nend\n"),
        ];
        assert_eq!(imports_of(&fragments, "driver< f = fill >").unwrap(), ["driver_lib::Driver", "fill_lib::Fill"]);
    }

    #[test]
    fn a_baked_type_imports_from_the_crate_that_declares_it_else_from_the_owner() {
        let fragments = [
            ("a_lib", "component baked\n  type Wrap<Leaf, Helper<Leaf>>\nend\n"),
            ("b_lib", "component leaf\n  type Leaf\nend\n"),
        ];
        // `Leaf` is a head in `b_lib`, `Helper` is declared by nobody.
        assert_eq!(imports_of(&fragments, "baked").unwrap(), ["a_lib::Helper", "a_lib::Wrap", "b_lib::Leaf"]);
    }

    #[test]
    fn holes_literals_and_keywords_import_nothing() {
        let c = [("k", "component c\n  type C<{n}, 16, true, usize>\n  const n values 1\nend\n")];
        assert_eq!(imports_of(&c, "c< n = 1 >").unwrap(), ["k::C"]);
    }

    #[test]
    fn a_listed_uses_replaces_the_derived_imports() {
        let c = [("k", "component c\n  type C<Foreign>\n  uses elsewhere::C elsewhere::Foreign\nend\n")];
        assert_eq!(imports_of(&c, "c").unwrap(), ["elsewhere::C", "elsewhere::Foreign"]);
    }

    #[test]
    fn a_component_with_neither_an_owning_crate_nor_uses_is_an_error() {
        let reg = Registry::parse("component c\n  type C\nend\n").unwrap();
        let err = resolve(&parse_spec("c").unwrap().1, &reg).unwrap_err();
        assert!(err.contains("`c` has no owning crate and no `uses`"), "got: {err}");
    }

    // A composite whose label opens with a slot hole, as `pk_single` does, is
    // grouped under its first filler's head, and that slot adds no axis of its own.
    const HOLE_FIRST: &str = "
component host
  type Host<{part}>
  label host<{part}>
  slot part Wrapped
end
component wrap
  type Wrap<{lead}, {tail}>
  label {lead}<{tail}>
  provides Wrapped
  slot lead Lead
  slot tail Tail
end
component pair
  type Pair<{inner}>
  label pair<{inner}>
  provides Lead
  slot inner Tail
end
component alpha
  type Alpha
  label alpha
  provides Lead
end
component beta
  type Beta
  label beta
  provides Tail
end
";

    fn axes_in_hole_first(spec: &str) -> Vec<(String, String, String)> {
        let reg = Registry::from_fragments([("lib", HOLE_FIRST)]).unwrap();
        let resolved = resolve(&parse_spec(spec).unwrap().1, &reg).unwrap();
        resolved.axes.into_iter().map(|a| (a.role, a.value, a.path)).collect()
    }

    #[test]
    fn a_label_opening_with_a_hole_takes_its_head_from_that_slot() {
        let axes = axes_in_hole_first("host< part = wrap< lead = alpha, tail = beta > >");
        assert_eq!(
            axes,
            vec![
                ("Wrapped".into(), "alpha".into(), "part".into()),
                ("Tail".into(), "beta".into(), "part/tail".into()),
            ]
        );
    }

    #[test]
    fn a_hole_first_head_keeps_the_sub_axes_of_its_first_filler() {
        let axes = axes_in_hole_first("host< part = wrap< lead = pair< inner = beta >, tail = beta > >");
        assert_eq!(
            axes,
            vec![
                ("Wrapped".into(), "pair".into(), "part".into()),
                ("Tail".into(), "beta".into(), "part/lead/inner".into()),
                ("Tail".into(), "beta".into(), "part/tail".into()),
            ]
        );
    }

    #[test]
    fn head_label_trims_to_the_component_head() {
        let reg = Registry::parse(REG).unwrap();
        // composite → head only
        assert_eq!(head_label(reg.get("combined").unwrap(), "combined<first, mid>", &[]), "combined");
        // leaf → full resolved label
        assert_eq!(head_label(reg.get("first_element").unwrap(), "first", &[]), "first");
    }
}
