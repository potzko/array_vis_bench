use lazy_static::lazy_static;
use std::sync::Mutex;

pub trait SortRegistry {
    fn register();
}

lazy_static! {
    /// Each entry: (registered_name, navigation_path).
    /// The path drives the tree menu; the name is passed to dispatch.
    static ref SORT_ENTRIES: Mutex<Vec<(String, Vec<String>)>> = Mutex::new(Vec::new());
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

/// Register a sort whose navigation path is derived from a raw category string.
///
/// Used by the `create_sort!` / `#[derive(SortRegistry)]` path, where the
/// caller passes `module_path!()` (e.g. `"array_vis_bench::sorts::bubble_sorts::bubble_sort"`).
/// The category is normalised to the sort-family name and the sort's own name
/// is appended, yielding a two-element path `[family, sort_name]`.
pub fn register_sort(name: &str, _big_o: &str, _stable: bool, category: &str) {
    let family = normalise_family(category);
    register_at_path(name, &[family.as_str(), name]);
}

/// Register a sort with an explicit navigation path.
///
/// Used by shell/comb/rod sort macros so they can place sorts at arbitrary
/// tree depth (e.g. `["shell sorts", "shell sort", "ciura"]`).
pub fn register_sort_path(name: &str, _big_o: &str, _stable: bool, path: &[&str]) {
    register_at_path(name, path);
}

fn register_at_path(name: &str, path: &[&str]) {
    let mut entries = SORT_ENTRIES.lock().unwrap();
    if !entries.iter().any(|(n, _)| n == name) {
        entries.push((name.to_string(), path.iter().map(|s| s.to_string()).collect()));
    }
}

/// Place an already-registered sort at an additional navigation path.
///
/// Unlike [`register_sort_path`], this does not deduplicate on `name` — the
/// same sort can appear at multiple positions in the navigation tree.  Only
/// adds a tree entry; does not touch `SORT_REGISTRY`.
pub fn register_tree_alias(name: &str, path: &[&str]) {
    let mut entries = SORT_ENTRIES.lock().unwrap();
    entries.push((name.to_string(), path.iter().map(|s| s.to_string()).collect()));
}

/// Normalise a raw category into a human-readable family name.
///
/// `module_path!()` → `"array_vis_bench::sorts::bubble_sorts::bubble_sort"`
/// Extract the `…_sorts` component and replace `_` with space.
fn normalise_family(raw: &str) -> String {
    if raw.contains("::") {
        raw.split("::")
            .find(|s| s.ends_with("_sorts"))
            .unwrap_or_else(|| { let parts: Vec<&str> = raw.split("::").collect(); parts.get(parts.len().saturating_sub(2)).copied().unwrap_or(raw) })
            .replace('_', " ")
    } else {
        raw.replace('_', " ")
    }
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

/// All registered sort names in registration order, deduplicated.
///
/// Deduplication is needed because the same name may appear multiple times
/// when registered at several tree positions via [`register_tree_alias`].
pub fn get_registered_sorts() -> Vec<String> {
    let entries = SORT_ENTRIES.lock().unwrap();
    let mut seen = std::collections::HashSet::new();
    entries
        .iter()
        .filter_map(|(name, _)| {
            if seen.insert(name.clone()) {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect()
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
}

/// Build the full navigation tree from all registered sorts.
pub fn get_sort_tree() -> SortTree {
    let entries = SORT_ENTRIES.lock().unwrap();
    let mut root = SortTree::default();
    for (name, path) in entries.iter() {
        root.insert(path, name);
    }
    root
}
