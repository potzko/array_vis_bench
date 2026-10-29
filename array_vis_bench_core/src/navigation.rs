//! The picker's view of `ALGORITHMS`, built on demand from each entry's `menu`,
//! `axes` and `label_template`. [`variants`] lists one [`VariantDesc`] per
//! entry for the faceted picker. [`SortTree`] nests the entries by menu path
//! and axis values, and `bench_registry::sorted` reads its depth first order.
//! [`repeated_name_errors`] backs the startup check that a name labels one entry.

use crate::bench_registry::{AlgorithmEntry, ALGORITHMS};
use std::collections::{BTreeMap, HashMap, HashSet};

/// One filled generic slot of a variant's type: a humanized trait label
/// (e.g. `"Pivot"`), the chosen value's head label (e.g. `"combined"`), and a
/// `path`, a unique `/` separated navigation key (e.g. `"pivot"`, `"pivot/a"`).
/// The path encodes nesting, so a composite filler (`combined<a, b>`) navigates
/// as its own level with sub-levels.
#[derive(Clone, Debug)]
pub struct AxisBinding {
    pub role: String,
    pub value: String,
    pub path: String,
}

/// One algorithm together with the structure the picker uses to place it: a
/// fixed `category` prefix and the faceted `axes`.
#[derive(Clone, Debug)]
pub struct VariantDesc {
    /// The entry's name, the key the picker looks it up by in `ALGORITHMS`.
    pub name: String,
    /// Structural nesting prefix, navigated in order before any axes.
    pub category: Vec<String>,
    /// Faceted axes in declaration order.
    pub axes: Vec<AxisBinding>,
    /// The label with one role tagged hole per axis, e.g.
    /// `"quick sort<part: {Partition}, small: {SmallSort}>"`. When present the
    /// picker renders the partial type in the catalog's own label syntax,
    /// filling each `{Role}` hole with the chosen value or `_`. `None` keeps the
    /// generic category and role breadcrumb.
    pub label_template: Option<String>,
}

impl From<&AlgorithmEntry> for VariantDesc {
    fn from(entry: &AlgorithmEntry) -> Self {
        VariantDesc {
            name: entry.name.to_string(),
            category: entry.menu.iter().map(|segment| segment.to_string()).collect(),
            axes: entry
                .axes
                .iter()
                .map(|&(role, value, path)| AxisBinding {
                    role: role.to_string(),
                    value: value.to_string(),
                    path: path.to_string(),
                })
                .collect(),
            label_template: (!entry.label_template.is_empty())
                .then(|| entry.label_template.to_string()),
        }
    }
}

/// One descriptor per registered entry, in `ALGORITHMS` order.
pub fn variants() -> Vec<VariantDesc> {
    ALGORITHMS.iter().map(VariantDesc::from).collect()
}

/// The entry's menu path followed by each axis value: the levels it occupies in
/// the flat navigation tree.
pub fn flat_path(entry: &AlgorithmEntry) -> Vec<&'static str> {
    let axis_values = entry.axes.iter().map(|&(_, value, _)| value);
    entry.menu.iter().copied().chain(axis_values).collect()
}

/// One message per name that labels more than one entry, listing each entry's
/// tree path. The picker looks a chosen name back up in `ALGORITHMS`, so a
/// shared name leaves all but one of its entries unreachable.
pub fn repeated_name_errors(
    entries: impl IntoIterator<Item = &'static AlgorithmEntry>,
) -> Vec<String> {
    let mut by_name: BTreeMap<&'static str, Vec<&'static AlgorithmEntry>> = BTreeMap::new();
    entries.into_iter().for_each(|entry| by_name.entry(entry.name).or_default().push(entry));
    by_name
        .into_iter()
        .filter(|(_, group)| group.len() > 1)
        .map(|(name, group)| {
            let paths: Vec<String> =
                group.iter().map(|entry| format!("[{}]", flat_path(entry).join("/"))).collect();
            format!(
                "algorithm '{name}' registered at {} distinct tree paths: {}",
                group.len(),
                paths.join(", ")
            )
        })
        .collect()
}

/// A node of the flat navigation tree, every level one menu segment or axis
/// value. Sub-trees sort by size then name and leaves by label then name, so
/// specialised (small group) entries surface first and ties are stable.
#[derive(Default)]
pub struct SortTree {
    /// Named sub-trees.
    pub children: Vec<(&'static str, SortTree)>,
    /// Entries ending at this level, keyed by their display label.
    pub leaves: Vec<(&'static str, &'static AlgorithmEntry)>,
}

impl SortTree {
    /// The sorted tree over `entries`.
    pub fn of(entries: impl IntoIterator<Item = &'static AlgorithmEntry>) -> SortTree {
        let mut root = SortTree::default();
        entries.into_iter().for_each(|entry| root.insert(&flat_path(entry), entry));
        root.sort_recursive();
        root
    }

    fn insert(&mut self, path: &[&'static str], entry: &'static AlgorithmEntry) {
        match path {
            [] => self.leaves.push(("", entry)),
            [label] => self.leaves.push((*label, entry)),
            [head, rest @ ..] => {
                let ind = self.children.iter().position(|(name, _)| name == head).unwrap_or_else(|| {
                    self.children.push((*head, SortTree::default()));
                    self.children.len() - 1
                });
                self.children[ind].1.insert(rest, entry);
            }
        }
    }

    /// Total number of leaves in this subtree.
    pub fn count_leaves(&self) -> usize {
        self.leaves.len() + self.children.iter().map(|(_, child)| child.count_leaves()).sum::<usize>()
    }

    fn sort_recursive(&mut self) {
        self.children.iter_mut().for_each(|(_, child)| child.sort_recursive());
        self.children.sort_by_cached_key(|(name, child)| (child.count_leaves(), *name));
        self.leaves.sort_by_key(|&(label, entry)| (label, entry.name));
    }

    /// Every entry in display order: depth first, the sub-trees of a level
    /// before its own leaves.
    pub fn entries(&self) -> Vec<&'static AlgorithmEntry> {
        let mut ret = Vec::with_capacity(self.count_leaves());
        self.push_entries(&mut ret);
        ret
    }

    fn push_entries(&self, out: &mut Vec<&'static AlgorithmEntry>) {
        self.children.iter().for_each(|(_, child)| child.push_entries(out));
        out.extend(self.leaves.iter().map(|&(_, entry)| entry));
    }
}

/// One axis fixed on the way to a leaf: the axis `path`, its display `role`
/// and the chosen `value`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pick {
    pub path: String,
    pub role: String,
    pub value: String,
}

/// The axes fixed so far, in the order they were picked.
pub type Chosen = Vec<Pick>;

/// One choice the picker offers: a branch narrowing the candidates, or an
/// entry to select.
pub enum Choice<'a> {
    Branch { label: String, cands: Vec<&'a VariantDesc>, depth: usize, chosen: Chosen },
    Leaf(&'a VariantDesc),
}

/// One step of the picker over its candidates: the choices to offer and what
/// the breadcrumb needs, the axis in focus as `(path, role)`, every other
/// unfixed axis as a `(path, role)` hole, and the label template when every
/// candidate shares one.
pub struct Step<'a> {
    pub choices: Vec<Choice<'a>>,
    pub axis: Option<(String, String)>,
    pub pending: Vec<(String, String)>,
    pub label_template: Option<&'a str>,
}

/// The picker's step function. `depth` is how many menu segments are consumed.
/// While a candidate still has a segment at `depth` this is structural
/// navigation, one branch per distinct segment. Past the menu it is faceted:
/// one axis at a time, the next unfixed path in declaration order (the type's
/// parameter order), never two axes in one list. Candidates with different
/// heads branch by head first, because one axis list cannot hold two heads:
/// the merge kinds share a menu, and the buffer axis of the auxiliary merges
/// would hide the rotation merges, which have no such axis.
pub fn step<'a>(cands: &[&'a VariantDesc], depth: usize, chosen: &Chosen) -> Step<'a> {
    let mut cat_order: Vec<String> = Vec::new();
    let mut cat_groups: HashMap<String, Vec<&VariantDesc>> = HashMap::new();
    let mut faceted: Vec<&VariantDesc> = Vec::new();
    for v in cands {
        if v.category.len() > depth {
            let key = v.category[depth].clone();
            if !cat_groups.contains_key(&key) { cat_order.push(key.clone()); }
            cat_groups.entry(key).or_default().push(*v);
        } else {
            faceted.push(*v);
        }
    }
    // An axis is fixed once its path has been chosen, keyed on path, not role,
    // so a composite's sub slots (`pivot/a`, `pivot/b`) stay distinct even
    // though they share the role `PivotSingle`.
    let chosen_paths: HashSet<&str> = chosen.iter().map(|p| p.path.as_str()).collect();
    let resolved = |v: &VariantDesc| v.axes.iter().all(|a| chosen_paths.contains(a.path.as_str()));
    let mut now = Step { choices: Vec::new(), axis: None, pending: Vec::new(), label_template: None };

    if !cat_groups.is_empty() {
        for key in &cat_order {
            let branch = Choice::Branch { label: key.clone(), cands: cat_groups[key].clone(), depth: depth + 1, chosen: chosen.clone() };
            now.choices.push(branch);
        }
        now.choices.extend(faceted.iter().copied().filter(|v| resolved(v)).map(Choice::Leaf));
        return now;
    }
    now.choices.extend(faceted.iter().copied().filter(|v| resolved(v)).map(Choice::Leaf));
    let axis_vars: Vec<&VariantDesc> = faceted.iter().copied().filter(|v| !resolved(v)).collect();
    let heads = head_groups(&axis_vars);
    if heads.len() > 1 {
        for (head, group) in heads {
            now.choices.push(Choice::Branch { label: head, cands: group, depth, chosen: chosen.clone() });
        }
        return now;
    }
    let Some((path, role, val_order, val_groups)) = next_facet(&axis_vars, &chosen_paths) else { return now };
    // Every still unfixed axis other than the one in focus, in pre order, shown
    // as holes so the partial type names all remaining parameters.
    for v in &axis_vars {
        for a in &v.axes {
            if !chosen_paths.contains(a.path.as_str()) && a.path != path && !now.pending.iter().any(|(p, _)| p == &a.path) {
                now.pending.push((a.path.clone(), a.role.clone()));
            }
        }
    }
    // The typed label rendering only fits the flat top level (its `{Role}`
    // holes do not nest); once navigation descends into a composite (a
    // `<slot>/...` path) the breadcrumb falls back to the generic form.
    let nested = path.contains('/') || chosen.iter().any(|p| p.path.contains('/'));
    if !nested {
        if let Some(first) = cands.first().and_then(|v| v.label_template.as_deref()) {
            if cands.iter().all(|v| v.label_template.as_deref() == Some(first)) { now.label_template = Some(first); }
        }
    }
    for val in &val_order {
        let mut sub = chosen.clone();
        sub.push(Pick { path: path.clone(), role: role.clone(), value: val.clone() });
        now.choices.push(Choice::Branch { label: val.clone(), cands: val_groups[val].clone(), depth, chosen: sub });
    }
    now.axis = Some((path, role));
    now
}

// The candidates grouped by head, the text of the label template before its
// `<`, in first appearance order. Candidates without a template form one group.
fn head_groups<'a>(axis_vars: &[&'a VariantDesc]) -> Vec<(String, Vec<&'a VariantDesc>)> {
    let head_of = |v: &VariantDesc| -> String {
        let template = v.label_template.as_deref().unwrap_or_default();
        template.split('<').next().unwrap_or(template).trim().to_string()
    };
    let mut groups: Vec<(String, Vec<&VariantDesc>)> = Vec::new();
    for v in axis_vars {
        let head = head_of(v);
        match groups.iter_mut().find(|(h, _)| *h == head) {
            Some((_, group)) => group.push(*v),
            None => groups.push((head, vec![*v])),
        }
    }
    groups
}

/// Every leaf the picker can reach from `cands`, walking every branch of every
/// step, in the order the picker would list them.
pub fn reachable<'a>(cands: &[&'a VariantDesc]) -> Vec<&'a str> {
    let mut out = Vec::new();
    walk(cands, 0, &Vec::new(), &mut out);
    out
}

fn walk<'a>(cands: &[&'a VariantDesc], depth: usize, chosen: &Chosen, out: &mut Vec<&'a str>) {
    for choice in step(cands, depth, chosen).choices {
        match choice {
            Choice::Branch { cands, depth, chosen, .. } => walk(&cands, depth, &chosen, out),
            Choice::Leaf(v) => out.push(v.name.as_str()),
        }
    }
}

// The next faceted step among `axis_vars` (candidates past the menu with at
// least one unfixed axis): the axis path to present, its display role, and the
// distinct values in pre order with the candidates carrying each. `None` once
// every axis is fixed.
#[allow(clippy::type_complexity)]
fn next_facet<'a>(
    axis_vars: &[&'a VariantDesc],
    chosen_paths: &HashSet<&str>,
) -> Option<(String, String, Vec<String>, HashMap<String, Vec<&'a VariantDesc>>)> {
    let path = choose_axis_path(axis_vars, chosen_paths)?;
    let role = axis_vars
        .iter()
        .find_map(|v| v.axes.iter().find(|a| a.path == path).map(|a| a.role.clone()))
        .unwrap_or_default();
    let mut val_order: Vec<String> = Vec::new();
    let mut val_groups: HashMap<String, Vec<&VariantDesc>> = HashMap::new();
    for v in axis_vars {
        if let Some(ab) = v.axes.iter().find(|a| a.path == path) {
            if !val_groups.contains_key(&ab.value) { val_order.push(ab.value.clone()); }
            val_groups.entry(ab.value.clone()).or_default().push(*v);
        }
    }
    Some((path, role, val_order, val_groups))
}

// The next axis to present: the first unfixed path in pre order, the order
// axes appear in each candidate's list, which mirrors the type's parameter
// tree (`QuickSort<Partition, Pivot, SmallSort>`: partition, pivot, a
// composite pivot's sub slots, small sort). A path present on every candidate
// is preferred so the list never mixes axes only some candidates have, which
// is also what defers a composite's sub slots (`pivot/a`) until its parent
// (`pivot`) is fixed to that composite.
fn choose_axis_path(axis_vars: &[&VariantDesc], chosen_paths: &HashSet<&str>) -> Option<String> {
    let n = axis_vars.len();
    let mut order: Vec<&str> = Vec::new();
    let mut present: HashMap<&str, usize> = HashMap::new();
    for v in axis_vars {
        for a in &v.axes {
            let path = a.path.as_str();
            if !chosen_paths.contains(path) {
                if !present.contains_key(path) { order.push(path); }
                *present.entry(path).or_default() += 1;
            }
        }
    }
    order.iter().find(|p| present[**p] == n).or_else(|| order.first()).map(|p| p.to_string())
}

#[cfg(test)]
mod picker_tests {
    use super::*;

    fn var(name: &str, axes: &[(&str, &str, &str)]) -> VariantDesc {
        VariantDesc {
            name: name.into(),
            category: vec!["sorts".into(), "quick sorts".into()],
            axes: axes
                .iter()
                .map(|(r, v, p)| AxisBinding { role: r.to_string(), value: v.to_string(), path: p.to_string() })
                .collect(),
            label_template: None,
        }
    }

    /// The structural nav: a composite filler (`combined<a,b>`) is its own level,
    /// then its sub slots `a` and `b` drill in, keyed on path, not role (both
    /// sub slots share role `PivotSingle`).
    #[test]
    fn combined_pivot_is_its_own_level_then_a_then_b() {
        let v1 = var("c1", &[
            ("Partition", "dual", "partition"), ("Pivot", "combined", "pivot"),
            ("PivotSingle", "first", "pivot/a"), ("PivotSingle", "first", "pivot/b"),
            ("SmallSort", "s", "small_sort"),
        ]);
        let v2 = var("c2", &[
            ("Partition", "dual", "partition"), ("Pivot", "combined", "pivot"),
            ("PivotSingle", "middle", "pivot/a"), ("PivotSingle", "first", "pivot/b"),
            ("SmallSort", "s", "small_sort"),
        ]);
        let vn = var("n", &[
            ("Partition", "dual", "partition"), ("Pivot", "ninther", "pivot"),
            ("SmallSort", "s", "small_sort"),
        ]);
        // After partition=dual, the pivot axis groups to {combined, ninther}: all
        // combined pairs collapse under the one head `combined`.
        let cands = [&v1, &v2, &vn];
        let chosen: HashSet<&str> = ["partition"].into_iter().collect();
        let (path, role, vals, groups) = next_facet(&cands, &chosen).unwrap();
        assert_eq!((path.as_str(), role.as_str()), ("pivot", "Pivot"));
        assert_eq!(vals, vec!["combined".to_string(), "ninther".to_string()]);
        assert_eq!(groups["combined"].len(), 2);
        // Descend into combined: the `a` sub slot is next, over {first, middle}.
        let combined = [&v1, &v2];
        let chosen2: HashSet<&str> = ["partition", "pivot"].into_iter().collect();
        let (path_a, _r, vals_a, _g) = next_facet(&combined, &chosen2).unwrap();
        assert_eq!(path_a, "pivot/a");
        assert_eq!(vals_a, vec!["first".to_string(), "middle".to_string()]);
    }

    fn kind(name: &str, template: &str, axis: (&str, &str, &str)) -> VariantDesc {
        VariantDesc { category: vec!["merges".into()], label_template: Some(template.into()), ..var(name, &[axis]) }
    }

    /// Two heads under one menu with different axes, the shape of the merge
    /// kinds: three auxiliary merges over a buffer and two rotation merges over
    /// a rotation. Every one of them must be reachable, and the first choice is
    /// the head, because one axis list cannot hold both.
    #[test]
    fn heads_with_different_axes_under_one_menu_are_all_reachable() {
        let kinds = [
            kind("auxiliary<full>", "auxiliary<{Auxiliary}>", ("Auxiliary", "full", "aux")),
            kind("auxiliary<none>", "auxiliary<{Auxiliary}>", ("Auxiliary", "none", "aux")),
            kind("naive<bridge>", "naive<{Rotation}>", ("Rotation", "bridge", "rot")),
            kind("naive<drill>", "naive<{Rotation}>", ("Rotation", "drill", "rot")),
            kind("smaller-side<drill>", "smaller-side<{Rotation}>", ("Rotation", "drill", "rot")),
        ];
        let cands: Vec<&VariantDesc> = kinds.iter().collect();
        let mut reached = reachable(&cands);
        reached.sort_unstable();
        let mut names: Vec<&str> = kinds.iter().map(|k| k.name.as_str()).collect();
        names.sort_unstable();
        assert_eq!(reached, names);
        let heads: Vec<String> = step(&cands, 1, &Vec::new())
            .choices
            .iter()
            .map(|c| match c { Choice::Branch { label, .. } => label.clone(), Choice::Leaf(v) => v.name.clone() })
            .collect();
        assert_eq!(heads, ["auxiliary", "naive", "smaller-side"]);
    }

    /// A single (leaf) pivot has no sub slots, so navigation goes straight from
    /// `pivot` to `small_sort`; `pivot/a` never appears.
    #[test]
    fn leaf_pivot_skips_to_small_sort() {
        let s = var("s", &[
            ("Partition", "left-left", "partition"), ("Pivot", "first", "pivot"),
            ("SmallSort", "size: 1", "small_sort"),
        ]);
        let cands = [&s];
        let chosen: HashSet<&str> = ["partition", "pivot"].into_iter().collect();
        let (path, _r, _v, _g) = next_facet(&cands, &chosen).unwrap();
        assert_eq!(path, "small_sort");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench_registry::Category;
    use array_vis_bench_traits::Complexity;

    macro_rules! entry {
        ($name:expr, $menu:expr, $axes:expr, $template:expr) => {
            AlgorithmEntry {
                name: $name,
                category: Category::Sort,
                worst: Complexity::CONST,
                best: Complexity::CONST,
                average: Complexity::CONST,
                space: Complexity::CONST,
                stable: true,
                adaptive: false,
                nondeterministic: false,
                max_input_size: None,
                menu: $menu,
                axes: $axes,
                label_template: $template,
                run: |_, _, _| {},
                run_with_input: |_, _, _| {},
                run_correctness: || {},
            }
        };
    }

    const QUICK: &[&str] = &["sorts", "quick sorts"];
    static LEFT: AlgorithmEntry =
        entry!("left", QUICK, &[("Partition", "left", "partition")], "q<{Partition}>");
    static RIGHT: AlgorithmEntry =
        entry!("right", QUICK, &[("Partition", "right", "partition")], "q<{Partition}>");
    static SOLO: AlgorithmEntry = entry!("solo", &["sorts", "cycle sorts"], &[], "");
    static TWIN_B: AlgorithmEntry = entry!("twin b", &["sorts", "twins"], &[], "");
    static TWIN_A: AlgorithmEntry = entry!("twin a", &["sorts", "twins"], &[], "");
    static LONE: AlgorithmEntry = entry!("lone", &["sorts", "alone", "inner"], &[], "");
    static SHARED_A: AlgorithmEntry = entry!("shared", &["sorts", "a"], &[], "");
    static SHARED_B: AlgorithmEntry =
        entry!("shared", &["sorts", "b"], &[("Role", "value", "role")], "");

    fn names(entries: &[&'static AlgorithmEntry]) -> Vec<&'static str> {
        entries.iter().map(|entry| entry.name).collect()
    }

    #[test]
    fn the_view_copies_the_entry_fields() {
        let view = VariantDesc::from(&LEFT);
        assert_eq!(view.name, "left");
        assert_eq!(view.category, ["sorts", "quick sorts"]);
        let axes: Vec<_> = view.axes.iter().map(|a| (&*a.role, &*a.value, &*a.path)).collect();
        assert_eq!(axes, [("Partition", "left", "partition")]);
        assert_eq!(view.label_template.as_deref(), Some("q<{Partition}>"));
    }

    #[test]
    fn an_empty_template_falls_back_to_the_generic_breadcrumb() {
        assert_eq!(VariantDesc::from(&SOLO).label_template, None);
    }

    #[test]
    fn the_flat_path_is_the_menu_then_the_axis_values() {
        assert_eq!(flat_path(&LEFT), ["sorts", "quick sorts", "left"]);
        assert_eq!(flat_path(&SOLO), ["sorts", "cycle sorts"]);
    }

    #[test]
    fn a_name_on_two_entries_reports_both_tree_paths() {
        let errors = repeated_name_errors([&LEFT, &SHARED_A, &SHARED_B, &SOLO]);
        assert_eq!(
            errors,
            ["algorithm 'shared' registered at 2 distinct tree paths: [sorts/a], [sorts/b/value]"]
        );
    }

    #[test]
    fn unique_names_report_nothing() {
        assert!(repeated_name_errors([&LEFT, &RIGHT, &SOLO]).is_empty());
    }

    #[test]
    fn sub_trees_sort_by_size_then_name() {
        let tree = SortTree::of([&RIGHT, &LEFT, &TWIN_A, &TWIN_B, &LONE]);
        let sorts = &tree.children[0].1;
        let order: Vec<_> = sorts.children.iter().map(|(name, child)| (*name, child.count_leaves())).collect();
        assert_eq!(order, [("alone", 1), ("quick sorts", 2)]);
    }

    #[test]
    fn leaves_sort_by_label_then_name() {
        let entries = SortTree::of([&TWIN_B, &TWIN_A]).entries();
        assert_eq!(names(&entries), ["twin a", "twin b"]);
    }

    #[test]
    fn sub_trees_come_before_the_leaves_of_their_level() {
        let entries = SortTree::of([&SOLO, &RIGHT, &LEFT]).entries();
        assert_eq!(names(&entries), ["left", "right", "solo"]);
    }
}
