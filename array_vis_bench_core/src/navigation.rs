//! The picker's view of `ALGORITHMS`, built on demand from each entry's `menu`,
//! `axes` and `label_template`. [`variants`] lists one [`VariantDesc`] per
//! entry for the faceted picker. [`SortTree`] nests the entries by menu path
//! and axis values, and `bench_registry::sorted` reads its depth first order.
//! [`repeated_name_errors`] backs the startup check that a name labels one entry.

use crate::bench_registry::{AlgorithmEntry, ALGORITHMS};
use std::collections::BTreeMap;

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
