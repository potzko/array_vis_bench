# Rod-sort family catalog fragment — OWNED by this crate, gathered by
# `spec_catalog` via `[package.metadata.array_vis_bench] spec = "rod.spec"`.
#
# Two-axis (uncoupled): one `RodSort<{strategy},{merge}>` driver whose two slots
# the query fills exhaustively. `strategy` ranges over the 6 BranchingStrategy
# impls, `merge` over the 2 RodMerge impls -> 6 x 2 = 12 entries. RodSort already
# has a unique type-head + inherent `sort` + composable trait impls, so it is
# queried directly (no driver wrapper). Component names are prefixed `rod_` for
# global uniqueness across the merged registry (Classic/Parity3/etc. clash with
# the shell family's own branching/sequence names). `uses` paths are plain text
# resolved in the consumer (`spec_catalog`).

# ════════════════════════════ DRIVER ════════════════════════════
component rod_sort
  type     RodSort<{strategy}, {merge}>
  label    rod sort<strategy: {strategy}, merge: {merge}>
  provides Sort
  category Sort
  menu     rod sorts
  uses     rod_sort_lib::RodSort
  slot     strategy RodBranching rod_classic
  slot     merge RodMerge rod_insertion
end

# ════════════════════════ BRANCHING STRATEGIES (6) ═══════════════════════
# labels match BranchingStrategy::NAME (used in the legacy menu path).
component rod_classic
  type     Classic
  label    classic
  provides RodBranching
  uses     rod_sort_lib::Classic
end

component rod_parity3
  type     Parity3
  label    3-parity
  provides RodBranching
  uses     rod_sort_lib::Parity3
end

component rod_log_parity
  type     LogParity
  label    log-parity
  provides RodBranching
  uses     rod_sort_lib::LogParity
end

component rod_root_parity
  type     RootParity
  label    root-parity
  provides RodBranching
  uses     rod_sort_lib::RootParity
end

component rod_optimised
  type     Optimised
  label    optimised
  provides RodBranching
  uses     rod_sort_lib::Optimised
end

component rod_fibonacci
  type     Fibonacci
  label    fibonacci
  provides RodBranching
  uses     rod_sort_lib::Fibonacci
end

# ════════════════════════ MERGE MODES (2) ═══════════════════════
# labels match RodMerge::NAME.
component rod_insertion
  type     InsertionMerge
  label    insertion
  provides RodMerge
  uses     rod_sort_lib::InsertionMerge
end

component rod_aux
  type     AuxMerge
  label    aux
  provides RodMerge
  uses     rod_sort_lib::AuxMerge
end
