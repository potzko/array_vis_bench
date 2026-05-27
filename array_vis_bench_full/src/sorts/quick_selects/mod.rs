pub mod quick_select;
// Dual-pivot quickselect is no longer a separate type family — it's
// `QuickSelect<DualPivotPartition, DPS>` through `quick_select_lib`. The old
// `dual_pivot_quick_select` shim is gone.
// `standalone_registry` moved to the sibling `quick_select_registry`
// crate; the wiring lib's `_QUICK_SELECT_REGISTRY_ANCHOR` keeps it
// linked. No re-export needed — the registry has no callable API.
