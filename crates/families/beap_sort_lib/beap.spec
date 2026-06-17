# Beap-sort family catalog fragment — OWNED by beap_sort_lib.
# Gathered via [package.metadata.array_vis_bench] spec = "beap.spec".
#
# Three driver heads (classic / quick-build single-pivot / quick-build dual-pivot)
# wrapping HeapSort<NaryHeapSort<BeapHeap<D>, DH>> with distinct type-heads. Beap
# REUSES heap.spec's shared roles HeapDir (2) + HeapPart (3) but has its OWN pivot
# axis (BeapPiv = 4, no ninther) and its own deep-heapify roles (beap-classic has
# only Iterative, vs heap-classic's 2), so those are beap-prefixed.
#
# Per-variant complexity is O(N^1.5) (N_SQRT_N) — the √N beap heapify — distinct
# from d-ary heap's N log N. Total: 2 + 72 + 6 = 80.

# ════════════════════════ BeapPiv (4, beap-only) ════════════════════════
component beap_v_first
  type     FirstElement
  label    first
  provides BeapPiv
  uses     pivot_first::FirstElement
end
component beap_v_middle
  type     MiddleElement
  label    middle
  provides BeapPiv
  uses     pivot_middle::MiddleElement
end
component beap_v_median3
  type     MedianOfThree
  label    median of 3
  provides BeapPiv
  uses     pivot_median3::MedianOfThree
end
component beap_v_median_of_medians
  type     MedianOfMedians
  label    median of medians
  provides BeapPiv
  uses     pivot_median_of_medians::MedianOfMedians
end

# ════════════════════════ DeepHeapify — classic (1, beap-only) ════════════════════════
component beap_dh_iterative
  type     Iterative
  label    iterative
  provides BeapDeepHeapifyClassic
  uses     heap_sort_lib::deep_heapify::Iterative
end

# ════════════════ DeepHeapify — quick single-pivot (3 composites, hp=HeapPart shared, v=BeapPiv) ════════════════
component beap_qdh_sequential
  type     SequentialQuickDeepHeapify<{hp}, {v}>
  label    sequential<part: {hp}, pivot: {v}>
  provides BeapDeepHeapifyQuick
  uses     heap_sort_lib::quick_deep_heapify::SequentialQuickDeepHeapify
  slot     hp HeapPart heap_part_left_left
  slot     v  BeapPiv  beap_v_first
end
component beap_qdh_recursive_partial
  type     RecursivePartialQuickDeepHeapify<{hp}, {v}>
  label    recursive partition<part: {hp}, pivot: {v}>
  provides BeapDeepHeapifyQuick
  uses     heap_sort_lib::quick_deep_heapify::RecursivePartialQuickDeepHeapify
  slot     hp HeapPart heap_part_left_left
  slot     v  BeapPiv  beap_v_first
end
component beap_qdh_stack_partial
  type     StackPartialQuickDeepHeapify<{hp}, {v}>
  label    stack partition<part: {hp}, pivot: {v}>
  provides BeapDeepHeapifyQuick
  uses     heap_sort_lib::quick_deep_heapify::StackPartialQuickDeepHeapify
  slot     hp HeapPart heap_part_left_left
  slot     v  BeapPiv  beap_v_first
end

# ════════════════ DeepHeapify — dual-pivot (1 composite over a DPS selector) ════════════════
component beap_dps_first
  type     CombinedSelector<FirstElement, FirstElement>
  label    first
  provides BeapDualPivot
  uses     quick_sort_lib::CombinedSelector pivot_first::FirstElement
end
component beap_dps_middle
  type     CombinedSelector<MiddleElement, MiddleElement>
  label    middle
  provides BeapDualPivot
  uses     quick_sort_lib::CombinedSelector pivot_middle::MiddleElement
end
component beap_dps_ninther
  type     NintherDualPivot
  label    ninther 1/3 + 2/3
  provides BeapDualPivot
  uses     quick_sort_lib::NintherDualPivot
end
component beap_dpqdh_stack_dual
  type     StackDualPivotPartialQuickDeepHeapify<{dps}>
  label    stack dual-pivot partition<dps: {dps}>
  provides BeapDeepHeapifyDual
  uses     heap_sort_lib::quick_deep_heapify::StackDualPivotPartialQuickDeepHeapify
  slot     dps BeapDualPivot beap_dps_first
end

# ════════════════════════ DRIVERS (3 distinct heads) ════════════════════════
component beap_sort_classic
  type     BeapSortClassicOf<{d}, {dh}>
  label    beap sort<dir: {d}, build: {dh}>
  provides Sort
  category Sort
  menu     heap sorts / beap / classic
  uses     beap_sort_lib::spec_drivers::BeapSortClassicOf
  slot     d  HeapDir                heap_dir_min_reverse
  slot     dh BeapDeepHeapifyClassic beap_dh_iterative
end
component beap_sort_quick_build
  type     BeapSortQuickOf<{d}, {dh}>
  label    beap sort quick build<dir: {d}, build: {dh}>
  provides Sort
  category Sort
  menu     heap sorts / beap / quick build / single pivot
  uses     beap_sort_lib::spec_drivers::BeapSortQuickOf
  slot     d  HeapDir              heap_dir_min_reverse
  slot     dh BeapDeepHeapifyQuick beap_qdh_sequential
end
component beap_sort_dual_build
  type     BeapSortDualOf<{d}, {dh}>
  label    beap sort quick build dual pivot<dir: {d}, build: {dh}>
  provides Sort
  category Sort
  menu     heap sorts / beap / quick build / dual pivot
  uses     beap_sort_lib::spec_drivers::BeapSortDualOf
  slot     d  HeapDir             heap_dir_min_reverse
  slot     dh BeapDeepHeapifyDual beap_dpqdh_stack_dual
end
