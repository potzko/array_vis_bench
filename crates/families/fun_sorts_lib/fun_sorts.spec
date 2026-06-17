# Fun-sorts family catalog fragment — OWNED by fun_sorts_lib.
#
# A grab bag of pedagogical / adversarial sorts. All `category Sort` (they sort an
# array); the interesting bit is `quick_surrender` / `quick_surrender_optimised`,
# which NEST the QuickSelect role (Phase 1's qs_recursive/qs_iterative) inside a
# sort driver — the inner select's partition/pivot arity is coupled in the query
# exactly as quick.spec couples a partition to its pivot.
#
# Super-quadratic complexity classes are real here (slow/potzko → O(2^N), stooge →
# O(N^2.71), cyclent → O(N^3)); the `Complexity` enum already represents them, and
# every emitted type carries its bounds via `fun_sorts_lib::composable`.
#
# `max_input` caps the (slow) chaos sorts so the correctness battery stays bounded
# — the contract-as-data form of the legacy `max_n_for_tests`. random shell sort is
# `nondeterministic` (randomised gaps) so it opts out of the determinism check.

# ════════════════════════════ ZERO-AXIS SORTS ═══════════════════════════════
component fs_slow_sort
  type      SlowSort
  label     slow sort
  provides  Sort
  category  Sort
  menu      fun sorts / slow sort
  uses      fun_sorts_lib::slow_sort::SlowSort
  max_input 150
end

component fs_slow_sort_potzko
  type      SlowSortPotzko
  label     slow sort potzko
  provides  Sort
  category  Sort
  menu      fun sorts / slow sort potzko
  uses      fun_sorts_lib::slow_sort_potzko::SlowSortPotzko
  max_input 20
end

component fs_bad_heap_sort
  type      BadHeapSort
  label     bad heap sort
  provides  Sort
  category  Sort
  menu      fun sorts / bad heap sort
  uses      fun_sorts_lib::bad_heap_sort::BadHeapSort
  max_input 200
end

component fs_bad_heap_sort_alt
  type     BadHeapSortAlt
  label    bad heap sort alt
  provides Sort
  category Sort
  menu     fun sorts / bad heap sort alt
  uses     fun_sorts_lib::bad_heap_sort_alt::BadHeapSortAlt
end

# ═════════════════════ CYCLENT PARTITIONS (curated, no MovingPivot) ══════════
# MovingPivot is deliberately excluded — it can leave the partition non-converged,
# which infinite-loops cyclent. These 4 are the proven-safe set.
component fs_part_left_left
  type     LeftLeftPartition
  label    left-left pointer
  provides CyclentPart
  uses     partition_lomuto::LeftLeftPartition
end

component fs_part_left_right
  type     LeftRightPartition
  label    left-right pointer
  provides CyclentPart
  uses     partition_hoare::LeftRightPartition
end

component fs_part_three_way
  type     ThreeWay
  label    three-way
  provides CyclentPart
  uses     partition_three_way::ThreeWay
end

component fs_part_block
  type     Block
  label    block
  provides CyclentPart
  uses     partition_block::Block
end

# ════════════════════════════ CYCLENT DRIVERS (×4 partitions) ════════════════
component fs_cyclent_sort
  type     CyclentSort<{part}>
  label    cyclent sort<{part}>
  provides Sort
  category Sort
  menu     fun sorts / cyclent sort
  uses     fun_sorts_lib::cyclent_sort::CyclentSort
  slot     part CyclentPart
end

component fs_cyclent_sort_opt
  type     CyclentSortOpt<{part}>
  label    cyclent sort opt<{part}>
  provides Sort
  category Sort
  menu     fun sorts / cyclent sort opt
  uses     fun_sorts_lib::cyclent_sort_opt::CyclentSortOpt
  slot     part CyclentPart
end

component fs_cyclent_sort_stack
  type     CyclentSortStack<{part}>
  label    cyclent sort stack<{part}>
  provides Sort
  category Sort
  menu     fun sorts / cyclent sort stack
  uses     fun_sorts_lib::cyclent_sort_stack::CyclentSortStack
  slot     part CyclentPart
end

component fs_cyclent_sort_stack_optimized
  type     CyclentSortStackOptimized<{part}>
  label    cyclent sort stack optimized<{part}>
  provides Sort
  category Sort
  menu     fun sorts / cyclent sort stack optimized
  uses     fun_sorts_lib::cyclent_sort_stack_optimized::CyclentSortStackOptimized
  slot     part CyclentPart
end

# ════════════════════════ NON-TRIVIAL SMALL SORTS (curated 2) ════════════════
# Used by stooge (bound SmallSort) and quick-surrender-optimised (bound
# NonTrivialSmallSort) — both types impl the needed trait.
component fs_ss_insertion32
  type     InsertionSmallSort<LinearInsertion, 32>
  label    insertion: 32
  provides FunSmallSort
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::LinearInsertion
end

component fs_ss_network16
  type     Network16SmallSort
  label    network: 16
  provides FunSmallSort
  uses     small_sort_network_16::Network16SmallSort
end

# ════════════════════════════ STOOGE (×2 small sorts) ════════════════════════
component fs_stooge_sort
  type      StoogeSort<{small}>
  label     stooge sort<{small}>
  provides  Sort
  category  Sort
  menu      fun sorts / stooge sort
  uses      fun_sorts_lib::stooge_sort::StoogeSort
  slot      small FunSmallSort
  max_input 500
end

# ════════════════════════ RANDOM SHELL (×5 gap distributions) ════════════════
component fs_random_shell_sort
  type            RandomShellSort<{dist}>
  label           random shell sort<{dist}>
  provides        Sort
  category        Sort
  menu            fun sorts / random shell sort
  uses            fun_sorts_lib::random_shell_sort::RandomShellSort
  slot            dist GapDistribution
  max_input       1000
  nondeterministic true
end

# ════════════════════════ QUICK SURRENDER (nests QuickSelect) ════════════════
# `inner` is filled by Phase 1's qs_recursive / qs_iterative (role QuickSelect);
# the query spells the nested fill with the arity-coupled partition/pivot vars.
component fs_quick_surrender
  type      QuickSurrender<{inner}>
  label     quick surrender<{inner}>
  provides  Sort
  category  Sort
  menu      fun sorts / quick surrender
  uses      fun_sorts_lib::quick_surrender::QuickSurrender
  slot      inner QuickSelect
  max_input 500
end

component fs_quick_surrender_optimised
  type      QuickSurrenderOptimised<{inner}, {small}>
  label     quick surrender optimised<{inner}, small: {small}>
  provides  Sort
  category  Sort
  menu      fun sorts / quick surrender optimised
  uses      fun_sorts_lib::quick_surrender_optimised::QuickSurrenderOptimised
  slot      inner QuickSelect
  slot      small FunSmallSort
  max_input 1000
end
