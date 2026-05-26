//! Navigation-tree registry for the CLI/menu surface.
//!
//! Holds the global list of `(name, path)` entries that leaf crates
//! register at process start via [`register_sort_path`]. The tree is
//! built lazily on demand by [`get_sort_tree`], which sorts each level
//! so smaller subtrees surface ahead of larger cross-products without
//! any caller-supplied ordering.
//!
//! This crate is intentionally minimal — no algorithm code, no logger,
//! no dep on the rest of the workspace except `lazy_static`. Any leaf
//! that wants a menu entry depends only on this crate (plus `ctor` for
//! the registration hook).

use lazy_static::lazy_static;
use std::sync::Mutex;

/// One row in the menu/registry: (registered name, navigation path).
///
/// Display order is derived from the resulting tree, not stored: each
/// `SortTree` level is sorted by subtree leaf count so specialised
/// (small-group) sorts surface ahead of large cross-products. There's no
/// need for callers to declare or even know the group size — it falls
/// out of how many siblings share the same parent path.
type Entry = (String, Vec<String>);

lazy_static! {
    static ref SORT_ENTRIES: Mutex<Vec<Entry>> = Mutex::new(Vec::new());
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

/// Register an algorithm with an explicit navigation path.
///
/// Used by `sort_family!`-generated code (and the rotation / partition /
/// small-sort registration macros) so each combination places itself at
/// arbitrary tree depth (e.g. `["sorts", "shell sorts", "ciura"]`).
///
/// Always appends — duplicate-name detection is the validator's job.
/// Silently dropping a re-registration here would let two different
/// algorithms share a name without anyone ever noticing; the validator
/// (called from `bench_registry::validate_registries`) surfaces them
/// at process start so misnamed entries fail loud.
pub fn register_sort_path(name: &str, _big_o: &str, _stable: bool, path: &[&str]) {
    let mut entries = SORT_ENTRIES.lock().unwrap();
    entries.push((name.to_string(), path.iter().map(|s| s.to_string()).collect()));
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

/// All registered sort names in display order (depth-first traversal of the
/// sorted tree). Each name appears at most once — the tree itself is the
/// dedup mechanism, since duplicate-name registrations are flagged by the
/// validator and never make it past process start in a valid build.
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

/// Snapshot of every `(name, path)` pair registered so far. The validator
/// in `bench_registry::validate_registries` uses this to detect names
/// registered more than once — silent dedup at registration time would
/// hide that, so we hand the raw list out and let the validator decide.
pub fn registered_path_entries() -> Vec<(String, Vec<String>)> {
    SORT_ENTRIES.lock().unwrap().clone()
}

/// A node in the sort navigation tree.
///
/// Each node has zero or more named sub-trees (`children`) and zero or more
/// leaf sorts (`leaves`).  At each level of the interactive menu the user
/// picks from the combined list of children + leaves.
#[derive(Default)]
pub struct SortTree {
    /// Named sub-trees, in insertion order.
    pub children: Vec<(String, SortTree)>,
    /// Leaf sorts at this level: `(display_label, registered_sort_name)`.
    pub leaves: Vec<(String, String)>,
}

impl SortTree {
    fn insert(&mut self, path: &[String], sort_name: &str) {
        if path.len() == 1 {
            self.leaves.push((path[0].clone(), sort_name.to_string()));
        } else {
            if let Some((_, child)) = self.children.iter_mut().find(|(k, _)| k == &path[0]) {
                child.insert(&path[1..], sort_name);
            } else {
                let mut child = SortTree::default();
                child.insert(&path[1..], sort_name);
                self.children.push((path[0].clone(), child));
            }
        }
    }

    /// Total number of leaves in this subtree (recursive).
    pub fn count_leaves(&self) -> usize {
        self.leaves.len()
            + self.children.iter().map(|(_, c)| c.count_leaves()).sum::<usize>()
    }

    /// Recursively reorder every level so smaller subtrees appear before
    /// larger ones. Children are ordered by `(subtree_size, label)`; leaves
    /// at the same level are ordered alphabetically by display label.
    /// Specialised (small-group) sorts naturally surface first.
    fn sort_recursive(&mut self) {
        for (_, child) in &mut self.children {
            child.sort_recursive();
        }
        self.children
            .sort_by(|a, b| (a.1.count_leaves(), &a.0).cmp(&(b.1.count_leaves(), &b.0)));
        self.leaves.sort_by(|a, b| a.0.cmp(&b.0));
    }
}

/// Build the full navigation tree from all registered sorts. Each level is
/// sorted by subtree size (smaller first) so specialised sorts surface
/// ahead of large cross-products without any caller-supplied ordering.
pub fn get_sort_tree() -> SortTree {
    let entries = SORT_ENTRIES.lock().unwrap();
    let mut root = SortTree::default();
    for (name, path) in entries.iter() {
        root.insert(path, name);
    }
    drop(entries);
    root.sort_recursive();
    root
}
