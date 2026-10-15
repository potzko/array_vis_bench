# `fun_sorts_lib`

Pedagogical and adversarial sorts: slow sort and its potzko variant, bad heap sort, four cyclent drivers over a partition scheme, stooge sort over a small sort, random shell sort over a gap distribution, and quick surrender over a nested quick select.

Catalog fragment `fun_sorts.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `fs_slow_sort` | `SlowSort` | `slow sort` | Sort | Sort |
| `fs_slow_sort_potzko` | `SlowSortPotzko` | `slow sort potzko` | Sort | Sort |
| `fs_bad_heap_sort` | `BadHeapSort` | `bad heap sort` | Sort | Sort |
| `fs_bad_heap_sort_alt` | `BadHeapSortAlt` | `bad heap sort alt` | Sort | Sort |
| `fs_cyclent_sort` | `CyclentSort<{part}>` | `cyclent sort<{part}>` | Sort | Sort |
| `fs_cyclent_sort_opt` | `CyclentSortOpt<{part}>` | `cyclent sort opt<{part}>` | Sort | Sort |
| `fs_cyclent_sort_stack` | `CyclentSortStack<{part}>` | `cyclent sort stack<{part}>` | Sort | Sort |
| `fs_cyclent_sort_stack_optimized` | `CyclentSortStackOptimized<{part}>` | `cyclent sort stack optimized<{part}>` | Sort | Sort |
| `fs_stooge_sort` | `StoogeSort<{small}>` | `stooge sort<{small}>` | Sort | Sort |
| `fs_random_shell_sort` | `RandomShellSort<{dist}>` | `random shell sort<{dist}>` | Sort | Sort |
| `fs_quick_surrender` | `QuickSurrender<{inner}>` | `quick surrender<{inner}>` | Sort | Sort |
| `fs_quick_surrender_optimised` | `QuickSurrenderOptimised<{inner}, {small}>` | `quick surrender optimised<{inner}, small: {small}>` | Sort | Sort |

Query `fun_sorts.avbs` builds: `SlowSorts`, `SlowSortPotzkos`, `BadHeapSorts`, `BadHeapSortAlts`, `CyclentSorts`, `CyclentSortOpts`, `CyclentSortStacks`, `CyclentSortStackOptimizeds`, `StoogeSorts`, `RandomShellSorts`, `qs_piv`, `QuickSurrenderRecPiv`, `QuickSurrenderItPiv`, `QuickSurrenderOptRecPiv`, `QuickSurrenderOptItPiv`, `he`, `QuickSurrenderRecHe`, `QuickSurrenderItHe`, `QuickSurrenderOptRecHe`, `QuickSurrenderOptItHe`.
