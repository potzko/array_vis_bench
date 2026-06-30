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
  category Sort
  menu     quick heap sorts / classic
  uses     quick_heap_sort_lib::quick_heap_sort::QuickHeapSort heap_sort_lib::deep_heapify::Iterative
  slot     arity HeapArity  heap_arity_binary
  slot     small SmallSort  ss_size1
end

component qhs_deferred
  type     DeferredQuickHeapSort<{arity}, {dss}>
  label    quick heap sort deferred<arity: {arity}, small: {dss}>
  provides Sort
  category Sort
  menu     quick heap sorts / deferred
  uses     quick_heap_sort_lib::deferred_quick_heap_sort::DeferredQuickHeapSort
  slot     arity HeapArity         heap_arity_binary
  slot     dss   DeferredSmallSort dss_lin16
end
component qs_part_he_bin_it
  type     HeapExtract<AryPair<Binary>, Iterative>
  label    heap extract: binary iterative
  provides QSPartition
  project  pivot QSPivotNone
  uses     quick_heap_sort_lib::heap_extract::HeapExtract quick_heap_sort_lib::heap_pair::AryPair heap_sort_lib::Binary heap_sort_lib::Iterative
end

component qs_part_he_bin_re
  type     HeapExtract<AryPair<Binary>, Recursive>
  label    heap extract: binary recursive
  provides QSPartition
  project  pivot QSPivotNone
  uses     quick_heap_sort_lib::heap_extract::HeapExtract quick_heap_sort_lib::heap_pair::AryPair heap_sort_lib::Binary heap_sort_lib::Recursive
end

component qs_part_he_ter_it
  type     HeapExtract<AryPair<Ternary>, Iterative>
  label    heap extract: ternary iterative
  provides QSPartition
  project  pivot QSPivotNone
  uses     quick_heap_sort_lib::heap_extract::HeapExtract quick_heap_sort_lib::heap_pair::AryPair heap_sort_lib::Ternary heap_sort_lib::Iterative
end

component qs_part_he_ter_re
  type     HeapExtract<AryPair<Ternary>, Recursive>
  label    heap extract: ternary recursive
  provides QSPartition
  project  pivot QSPivotNone
  uses     quick_heap_sort_lib::heap_extract::HeapExtract quick_heap_sort_lib::heap_pair::AryPair heap_sort_lib::Ternary heap_sort_lib::Recursive
end
