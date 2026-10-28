# Quick-heap-sort family catalog fragment — OWNED by quick_heap_sort_lib.
# Gathered via [package.metadata.array_vis_bench] spec = "quick_heap.spec".
#
# PORTABLE SUBSET: the clean `classic` (QuickHeapSort<A, Iterative, SS>) and
# `deferred` (DeferredQuickHeapSort<A, DSS>) families — both have a unique
# type-head, an inherent `sort`, and the full composable triad. Total 32.
#
# The quick-build / heap-extract / pivotless-cycle families are DEFERRED: they
# rely on per-component `max_visits` recursion budgets which the .spec grammar
# does not express, and bound a PivotlessPartition<->HeapExtractBuild cycle that
# global depth-limiting cannot reproduce without blow-up. (See memory note.)
#
# These two families reuse roles already in the merged registry — HeapArity
# (owned by heap.spec) and SmallSort / DeferredSmallSort (owned by quick.spec) —
# so this fragment declares only the two driver components.

component qhs_classic
  type     QuickHeapSort<{arity}, Iterative, {small}>
  label    quick heap sort<arity: {arity}, small: {small}>
  provides Sort
  menu     quick heap sorts / classic
  slot     arity HeapArity
  slot     small SmallSort
end

component qhs_deferred
  type     DeferredQuickHeapSort<{arity}, {dss}>
  label    quick heap sort deferred<arity: {arity}, small: {dss}>
  provides Sort
  menu     quick heap sorts / deferred
  slot     arity HeapArity
  slot     dss   DeferredSmallSort
end
# ═══════════════════ HEAP-EXTRACT PARTITIONS (pivotless) ═══════════════════
# Self-pivoting `PartitionScheme`s (N_PIVOTS = 0) — no pivot to select. Under the
# unified partition role-system they provide `HeapExtractPart` (their isolation role,
# so a `he: HeapExtractPart = _` hole enumerates exactly these 4) AND `SelectPartition`
# (quick-select's `part` slot role, shared with the pivoted composites). They do NOT
# provide `PartitionKind`, so quick and the standalone Partition kind never see them —
# only quick-select unions them into its `part` set.
component qs_part_he_bin_it
  type     HeapExtract<AryPair<Binary>, Iterative>
  label    heap extract: binary iterative
  provides HeapExtractPart SelectPartition
end

component qs_part_he_bin_re
  type     HeapExtract<AryPair<Binary>, Recursive>
  label    heap extract: binary recursive
  provides HeapExtractPart SelectPartition
end

component qs_part_he_ter_it
  type     HeapExtract<AryPair<Ternary>, Iterative>
  label    heap extract: ternary iterative
  provides HeapExtractPart SelectPartition
end

component qs_part_he_ter_re
  type     HeapExtract<AryPair<Ternary>, Recursive>
  label    heap extract: ternary recursive
  provides HeapExtractPart SelectPartition
end

# ═══════════ RECURSIVE SET-QUICK-SELECT DEEP-HEAPIFY (finite() dogfood) ═══════════
# A GENUINELY RECURSIVE role graph — the deep-heapify of a heap is itself driven by a
# quick-select whose partition is an open heap-extract whose OWN deep-heapify recurses:
#     SetDeepHeapify → RecDeepHeapify(set arm) → RecSelect → RecSelPart(hx_open)
#                    → RecDeepHeapify → …
# `RecDeepHeapify` reaches itself, so a bare `_` blows up (budget-refused). The driver
# query bounds it with `finite( _ )`: the occurs check forbids `rdh_set` / `hx_open`
# from nesting in themselves, so the cycle unfolds EXACTLY ONCE and terminates at a
# leaf deep-heapify. The driver's `dh` slot is the SET-ONLY `SetDeepHeapify` (so only
# the recursive arm is emitted — the Iterative/Recursive leaves are the classic family,
# not re-emitted here). finite() emits the 2 bounded variants; without it, nothing.
component rdh_set
  type     SequentialSet<{qs}>
  label    set<{qs}>
  provides SetDeepHeapify
  # Listed rather than derived: SequentialSet lives in heap_sort_lib, not in the owning crate.
  uses     heap_sort_lib::SequentialSet
  slot     qs RecSelect
end
component rsel
  type     SetSelect<{part}>
  label    select<{part}>
  provides RecSelect
  # Listed rather than derived: SetSelect lives in quick_select_lib, not in the owning crate.
  uses     quick_select_lib::SetSelect
  slot     part RecSelPart
end
# The classic leaves plus the set arm, so the open heap-extract nests either.
role RecDeepHeapify = DeepHeapifyClassic | SetDeepHeapify
component hx_open
  type     HeapExtract<AryPair<Binary>, {dh}>
  label    heap extract: binary<{dh}>
  provides RecSelPart
  slot     dh RecDeepHeapify
end
component set_quick_heap_sort
  type     SetQuickHeapSort<{dh}>
  label    set quick heap sort<{dh}>
  provides Sort
  menu     quick heap sorts / set
  slot     dh SetDeepHeapify
  max_input 128
end
