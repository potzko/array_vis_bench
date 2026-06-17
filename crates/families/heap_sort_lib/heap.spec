# Heap-sort family catalog fragment — OWNED by heap_sort_lib, gathered by
# `spec_catalog` via [package.metadata.array_vis_bench] spec = "heap.spec".
#
# Three driver heads (classic d-ary / quickselect-build single-pivot / quickselect-
# build dual-pivot) over the shared Arity x Direction facets. The drivers wrap the
# real HeapSort<NaryHeapSort<ArityHeap<A,D>, DH>> chain with unique type-heads (the
# legacy types all share the `HeapSort` head, which AVBS resolves first-wins).
#
# This fragment OWNS the heap-cluster's shared role components HeapDir (2) and
# HeapPart (3) — weak_heap and beap REFERENCE these roles (do not redeclare).
# Everything else (HeapArity, HeapPiv, DeepHeapify*) is heap-specific.

# ════════════════════════ Arity (4) ════════════════════════
component heap_arity_binary
  type     Binary
  label    binary
  provides HeapArity
  uses     heap_sort_lib::arity::Binary
end
component heap_arity_ternary
  type     Ternary
  label    ternary
  provides HeapArity
  uses     heap_sort_lib::arity::Ternary
end
component heap_arity_16
  type     Base16
  label    16-ary
  provides HeapArity
  uses     heap_sort_lib::arity::Base16
end
component heap_arity_256
  type     Base256
  label    256-ary
  provides HeapArity
  uses     heap_sort_lib::arity::Base256
end

# ════════════════════════ HeapDir (2, SHARED) ════════════════════════
# Only the two ascending-producing directions carry the marker (legacy parity).
component heap_dir_min_reverse
  type     MinReverse
  label    min reverse
  provides HeapDir
  uses     heap_sort_lib::direction::MinReverse
end
component heap_dir_max_forward
  type     MaxForward
  label    max forward
  provides HeapDir
  uses     heap_sort_lib::direction::MaxForward
end

# ════════════════════════ HeapPart (3, SHARED) ════════════════════════
component heap_part_left_left
  type     LeftLeftPartition
  label    left-left pointer
  provides HeapPart
  uses     heap_sort_lib::heap_partition::LeftLeftPartition
end
component heap_part_left_right
  type     LeftRightPartition
  label    left-right pointer
  provides HeapPart
  uses     heap_sort_lib::heap_partition::LeftRightPartition
end
component heap_part_block
  type     Block
  label    block
  provides HeapPart
  uses     heap_sort_lib::heap_partition::Block
end

# ════════════════════════ HeapPiv (5, heap-only) ════════════════════════
component heap_piv_first
  type     FirstElement
  label    first
  provides HeapPiv
  uses     pivot_first::FirstElement
end
component heap_piv_middle
  type     MiddleElement
  label    middle
  provides HeapPiv
  uses     pivot_middle::MiddleElement
end
component heap_piv_median3
  type     MedianOfThree
  label    median of 3
  provides HeapPiv
  uses     pivot_median3::MedianOfThree
end
component heap_piv_median_of_medians
  type     MedianOfMedians
  label    median of medians
  provides HeapPiv
  uses     pivot_median_of_medians::MedianOfMedians
end
component heap_piv_ninther
  type     Ninther
  label    ninther
  provides HeapPiv
  uses     pivot_ninther::Ninther
end

# ════════════════════════ DeepHeapifyClassic (2) ════════════════════════
component heap_dh_iterative
  type     Iterative
  label    iterative
  provides DeepHeapifyClassic
  uses     heap_sort_lib::deep_heapify::Iterative
end
component heap_dh_recursive
  type     Recursive
  label    recursive
  provides DeepHeapifyClassic
  uses     heap_sort_lib::deep_heapify::Recursive
end

# ════════════════ DeepHeapifyQuick (3 composites, each <HP, V>) ════════════════
component heap_qdh_sequential
  type     SequentialQuickDeepHeapify<{hp}, {v}>
  label    sequential<part: {hp}, pivot: {v}>
  provides DeepHeapifyQuick
  uses     heap_sort_lib::quick_deep_heapify::SequentialQuickDeepHeapify
  slot     hp HeapPart heap_part_left_left
  slot     v  HeapPiv  heap_piv_first
end
component heap_qdh_recursive_partial
  type     RecursivePartialQuickDeepHeapify<{hp}, {v}>
  label    recursive partition<part: {hp}, pivot: {v}>
  provides DeepHeapifyQuick
  uses     heap_sort_lib::quick_deep_heapify::RecursivePartialQuickDeepHeapify
  slot     hp HeapPart heap_part_left_left
  slot     v  HeapPiv  heap_piv_first
end
component heap_qdh_stack_partial
  type     StackPartialQuickDeepHeapify<{hp}, {v}>
  label    stack partition<part: {hp}, pivot: {v}>
  provides DeepHeapifyQuick
  uses     heap_sort_lib::quick_deep_heapify::StackPartialQuickDeepHeapify
  slot     hp HeapPart heap_part_left_left
  slot     v  HeapPiv  heap_piv_first
end

# ════════════════ DeepHeapifyDual (3 dual-pivot selectors) ════════════════
component heap_dps_first
  type     StackDualPivotPartialQuickDeepHeapify<CombinedSelector<FirstElement, FirstElement>>
  label    stack dual-pivot partition<first>
  provides DeepHeapifyDual
  uses     heap_sort_lib::quick_deep_heapify::StackDualPivotPartialQuickDeepHeapify quick_sort_lib::CombinedSelector pivot_first::FirstElement
end
component heap_dps_middle
  type     StackDualPivotPartialQuickDeepHeapify<CombinedSelector<MiddleElement, MiddleElement>>
  label    stack dual-pivot partition<middle>
  provides DeepHeapifyDual
  uses     heap_sort_lib::quick_deep_heapify::StackDualPivotPartialQuickDeepHeapify quick_sort_lib::CombinedSelector pivot_middle::MiddleElement
end
component heap_dps_ninther
  type     StackDualPivotPartialQuickDeepHeapify<NintherDualPivot>
  label    stack dual-pivot partition<ninther 1/3 + 2/3>
  provides DeepHeapifyDual
  uses     heap_sort_lib::quick_deep_heapify::StackDualPivotPartialQuickDeepHeapify quick_sort_lib::NintherDualPivot
end

# ════════════════════════ DRIVERS (3 unique heads) ════════════════════════
component heap_sort_classic
  type     HeapSortClassicOf<{a}, {d}, {dh}>
  label    heap sort<arity: {a}, dir: {d}, build: {dh}>
  provides Sort
  category Sort
  menu     heap sorts / d-ary / classic
  uses     heap_sort_lib::spec_drivers::HeapSortClassicOf
  slot     a  HeapArity          heap_arity_binary
  slot     d  HeapDir            heap_dir_min_reverse
  slot     dh DeepHeapifyClassic heap_dh_iterative
end
component heap_sort_quick_build
  type     HeapSortQuickBuildOf<{a}, {d}, {dh}>
  label    heap sort quick build<arity: {a}, dir: {d}, build: {dh}>
  provides Sort
  category Sort
  menu     heap sorts / d-ary / quick build / single pivot
  uses     heap_sort_lib::spec_drivers::HeapSortQuickBuildOf
  slot     a  HeapArity        heap_arity_binary
  slot     d  HeapDir          heap_dir_min_reverse
  slot     dh DeepHeapifyQuick heap_qdh_sequential
end
component heap_sort_dual_build
  type     HeapSortDualBuildOf<{a}, {d}, {dh}>
  label    heap sort quick build dual pivot<arity: {a}, dir: {d}, build: {dh}>
  provides Sort
  category Sort
  menu     heap sorts / d-ary / quick build / dual pivot
  uses     heap_sort_lib::spec_drivers::HeapSortDualBuildOf
  slot     a  HeapArity       heap_arity_binary
  slot     d  HeapDir         heap_dir_min_reverse
  slot     dh DeepHeapifyDual heap_dps_first
end
