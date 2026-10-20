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
  menu     rod sorts
  slot     strategy RodBranching
  slot     merge RodMerge
end

# ════════════════════════ BRANCHING STRATEGIES (6) ═══════════════════════
# labels match BranchingStrategy::NAME.
component rod_classic
  type     Classic
  label    classic
  provides RodBranching
end

component rod_parity3
  type     Parity3
  label    3-parity
  provides RodBranching
end

component rod_log_parity
  type     LogParity
  label    log-parity
  provides RodBranching
end

component rod_root_parity
  type     RootParity
  label    root-parity
  provides RodBranching
end

component rod_optimised
  type     Optimised
  label    optimised
  provides RodBranching
end

component rod_fibonacci
  type     Fibonacci
  label    fibonacci
  provides RodBranching
end

# ════════════════════════ MERGE MODES (2) ═══════════════════════
# labels match RodMerge::NAME.
component rod_insertion
  type     InsertionMerge
  label    insertion
  provides RodMerge
end

component rod_aux
  type     AuxMerge
  label    aux
  provides RodMerge
end
