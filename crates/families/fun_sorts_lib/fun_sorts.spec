# Fun-sorts family catalog fragment — OWNED by fun_sorts_lib.
#
# A grab bag of pedagogical / adversarial sorts. All are `SortingAlgorithm`
# entries (they sort an array); the interesting bit is `quick_surrender` / `quick_surrender_optimised`,
# which NEST the QuickSelect role (Phase 1's qs_recursive/qs_iterative) inside a
# sort driver — the inner select's partition/pivot arity is coupled in the query
# exactly as quick.spec couples a partition to its pivot.
#
# Super-quadratic complexity classes are real here (slow/potzko → O(2^N), stooge →
# O(N^2.71), cyclent → O(N^3)); the `Complexity` enum already represents them, and
# every emitted type carries its bounds via `fun_sorts_lib::composable`.
#
# `max_input` caps the (slow) chaos sorts so the correctness battery stays bounded.
# random shell sort is `nondeterministic` (randomised gaps), so a check that compares
# two traces of one entry skips it.

# ════════════════════════════ ZERO-AXIS SORTS ═══════════════════════════════
component fs_slow_sort
  type      SlowSort
  label     slow sort
  provides  Sort
  menu      fun sorts / slow sort
  max_input 150
end

component fs_slow_sort_potzko
  type      SlowSortPotzko
  label     slow sort potzko
  provides  Sort
  menu      fun sorts / slow sort potzko
  max_input 20
end

component fs_bad_heap_sort
  type      BadHeapSort
  label     bad heap sort
  provides  Sort
  menu      fun sorts / bad heap sort
  max_input 200
end

component fs_bad_heap_sort_alt
  type     BadHeapSortAlt
  label    bad heap sort alt
  provides Sort
  menu     fun sorts / bad heap sort alt
end

# ═════════════════════ CYCLENT PARTITIONS (curated, no MovingPivot) ══════════
# MovingPivot is deliberately excluded — it can leave the partition non-converged,
# which infinite-loops cyclent. These 4 are the proven-safe set.

# ════════════════════════════ CYCLENT DRIVERS (×4 partitions) ════════════════
component fs_cyclent_sort
  type     CyclentSort<{part}>
  label    cyclent sort<{part}>
  provides Sort
  menu     fun sorts / cyclent sort
  slot     part PartScheme
  max_input 5000
end

component fs_cyclent_sort_opt
  type     CyclentSortOpt<{part}>
  label    cyclent sort opt<{part}>
  provides Sort
  menu     fun sorts / cyclent sort opt
  slot     part PartScheme
  max_input 5000
end

component fs_cyclent_sort_stack
  type     CyclentSortStack<{part}>
  label    cyclent sort stack<{part}>
  provides Sort
  menu     fun sorts / cyclent sort stack
  slot     part PartScheme
  max_input 5000
end

component fs_cyclent_sort_stack_optimized
  type     CyclentSortStackOptimized<{part}>
  label    cyclent sort stack optimized<{part}>
  provides Sort
  menu     fun sorts / cyclent sort stack optimized
  slot     part PartScheme
  max_input 5000
end

# ════════════════════════ NON-TRIVIAL SMALL SORTS (curated 2) ════════════════
# Used by stooge (bound SmallSort) and quick-surrender-optimised (bound
# NonTrivialSmallSort) — both types impl the needed trait.
role FunSmallSort = SmallSort - ss_size1 - ss_size2

# ════════════════════════════ STOOGE (×2 small sorts) ════════════════════════
component fs_stooge_sort
  type      StoogeSort<{small}>
  label     stooge sort<{small}>
  provides  Sort
  menu      fun sorts / stooge sort
  slot      small FunSmallSort
  max_input 500
end

# ════════════════════════ RANDOM SHELL (×5 gap distributions) ════════════════
component fs_random_shell_sort
  type            RandomShellSort<{dist}>
  label           random shell sort<{dist}>
  provides        Sort
  menu            fun sorts / random shell sort
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
  menu      fun sorts / quick surrender
  slot      inner QuickSelect
  max_input 500
end

component fs_quick_surrender_optimised
  type      QuickSurrenderOptimised<{inner}, {small}>
  label     quick surrender optimised<{inner}, small: {small}>
  provides  Sort
  menu      fun sorts / quick surrender optimised
  slot      inner QuickSelect
  slot      small FunSmallSort
  max_input 1000
end
