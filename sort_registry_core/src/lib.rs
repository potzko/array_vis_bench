//! Navigation registry for the CLI/menu surface.
//!
//! Holds the global list of variant descriptors that leaf crates register
//! at process start. Each descriptor carries a structural **category**
//! prefix (e.g. `["sorts", "quick sorts"]`) plus a list of **axes** —
//! `(role, value)` pairs describing the type's filled generic slots
//! (e.g. `("partition", "left-left pointer")`). The interactive picker navigates the
//! category levels as a fixed tree, then presents the axes one role at a
//! time (faceted navigation).
//!
//! Two registration entry points:
//!   * [`register_sort_variant`] — structured: category + axes. Emitted by
//!     `sort_family!`-generated code.
//!   * [`register_sort_path`] — legacy flat path (no axes). Used by the
//!     hand-written rotation / partition / small-sort / circle macros. The
//!     whole path becomes the category; the variant has no faceted axes.
//!
//! This crate is intentionally minimal — no algorithm code, no logger,
//! no dep on the rest of the workspace except `lazy_static`.

use lazy_static::lazy_static;
use std::sync::Mutex;

/// One filled generic slot of a variant's type: a humanized trait/role
/// label (e.g. `"partition"`) and the chosen value's label (e.g.
/// `"left-left pointer"`).
#[derive(Clone, Debug)]
pub struct AxisBinding {
    pub role: String,
    pub value: String,
}

/// A single registered algorithm together with the structure the picker
/// uses to place it: a fixed `category` prefix and the faceted `axes`.
#[derive(Clone, Debug)]
pub struct VariantDesc {
    /// Registered algorithm name (the leaf identifier looked up in
    /// `ALGORITHMS`).
    pub name: String,
    /// Structural nesting prefix, navigated in order before any axes.
    pub category: Vec<String>,
    /// Faceted axes in declaration order. Empty for legacy flat-path
    /// registrations.
    pub axes: Vec<AxisBinding>,
}

impl VariantDesc {
    /// Flattened navigation path = category followed by each axis value.
    /// Used for the legacy tree view and the duplicate-name validator.
    pub fn flat_path(&self) -> Vec<String> {
        let mut p = self.category.clone();
        p.extend(self.axes.iter().map(|a| a.value.clone()));
        p
    }
}

lazy_static! {
    static ref VARIANTS: Mutex<Vec<VariantDesc>> = Mutex::new(Vec::new());
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

/// Register a structured variant: a category prefix plus faceted axes.
///
/// `axes` is `(role_label, value_label)` in declaration order. The picker
/// reorders them at display time (largest axis first), so declaration
/// order only breaks ties.
pub fn register_sort_variant(name: &str, category: &[&str], axes: &[(&str, &str)]) {
    let mut entries = VARIANTS.lock().unwrap();
    entries.push(VariantDesc {
        name: name.to_string(),
        category: category.iter().map(|s| s.to_string()).collect(),
        axes: axes
            .iter()
            .map(|(role, value)| AxisBinding {
                role: role.to_string(),
                value: value.to_string(),
            })
            .collect(),
    });
}

/// Register an algorithm with an explicit flat navigation path and no
/// faceted axes. The whole path is treated as the category prefix.
///
/// Used by the hand-written rotation / partition / small-sort / circle
/// registration macros. Always appends — duplicate-name detection is the
/// validator's job.
pub fn register_sort_path(name: &str, _big_o: &str, _stable: bool, path: &[&str]) {
    let mut entries = VARIANTS.lock().unwrap();
    entries.push(VariantDesc {
        name: name.to_string(),
        category: path.iter().map(|s| s.to_string()).collect(),
        axes: Vec::new(),
    });
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

/// Snapshot of every registered variant descriptor. The picker consumes
/// this directly for faceted navigation.
pub fn all_variants() -> Vec<VariantDesc> {
    VARIANTS.lock().unwrap().clone()
}

/// All registered sort names in display order (depth-first traversal of the
/// sorted tree). Each name appears at most once.
pub fn get_registered_sorts() -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    flatten_tree(&get_sort_tree(), &mut out, &mut seen);
    out
}

fn flatten_tree(
    tree: &SortTree,
    out: &mut Vec<String>,
    seen: &mut std::collections::HashSet<String>,
) {
    for (_, child) in &tree.children {
        flatten_tree(child, out, seen);
    }
    for (_, name) in &tree.leaves {
        if seen.insert(name.clone()) {
            out.push(name.clone());
        }
    }
}

/// Snapshot of every `(name, flat_path)` pair registered so far. The
/// validator in `bench_registry::validate_registries` uses this to detect
/// names registered more than once.
pub fn registered_path_entries() -> Vec<(String, Vec<String>)> {
    VARIANTS
        .lock()
        .unwrap()
        .iter()
        .map(|v| (v.name.clone(), v.flat_path()))
        .collect()
}

/// A node in the sort navigation tree (legacy flat view).
///
/// Retained for `get_registered_sorts` / the validator and any consumer
/// that wants the pre-faceting flat tree. The interactive picker uses
/// [`all_variants`] + faceted navigation instead.
#[derive(Default)]
pub struct SortTree {
    /// Named sub-trees, in insertion order.
    pub children: Vec<(String, SortTree)>,
    /// Leaf sorts at this level: `(display_label, registered_sort_name)`.
    pub leaves: Vec<(String, String)>,
}

impl SortTree {
    fn insert(&mut self, path: &[String], sort_name: &str) {
        if path.len() <= 1 {
            let label = path.first().cloned().unwrap_or_default();
            self.leaves.push((label, sort_name.to_string()));
        } else if let Some((_, child)) =
            self.children.iter_mut().find(|(k, _)| k == &path[0])
        {
            child.insert(&path[1..], sort_name);
        } else {
            let mut child = SortTree::default();
            child.insert(&path[1..], sort_name);
            self.children.push((path[0].clone(), child));
        }
    }

    /// Total number of leaves in this subtree (recursive).
    pub fn count_leaves(&self) -> usize {
        self.leaves.len()
            + self.children.iter().map(|(_, c)| c.count_leaves()).sum::<usize>()
    }

    fn sort_recursive(&mut self) {
        for (_, child) in &mut self.children {
            child.sort_recursive();
        }
        self.children
            .sort_by(|a, b| (a.1.count_leaves(), &a.0).cmp(&(b.1.count_leaves(), &b.0)));
        self.leaves.sort_by(|a, b| a.0.cmp(&b.0));
    }
}

/// Build the legacy flat navigation tree from all registered variants
/// (category + axis values joined into one path).
pub fn get_sort_tree() -> SortTree {
    let entries = VARIANTS.lock().unwrap();
    let mut root = SortTree::default();
    for v in entries.iter() {
        root.insert(&v.flat_path(), &v.name);
    }
    drop(entries);
    root.sort_recursive();
    root
}
