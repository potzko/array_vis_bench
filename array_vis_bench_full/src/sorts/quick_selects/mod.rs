pub mod quick_select;
pub mod dual_pivot_quick_select;
// `standalone_registry` moved to the sibling `quick_select_registry`
// crate; the wiring lib's `_QUICK_SELECT_REGISTRY_ANCHOR` keeps it
// linked. No re-export needed — the registry has no callable API.
