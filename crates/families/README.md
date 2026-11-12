# families

Drivers and composers. A family crate holds the generic sort types, their
annotation impls, a `.spec` fragment declaring the drivers and any family
local fillers, and the `.avbs` query that builds the family.

- `beap_sort_lib`: beap sort over the heap sort driver.
- `block_merge_sort_lib`: block merge sort over a buffer and a key region, the WikiSort and GrailSort shapes.
- `bubble_sort_lib`: bubble, shaker, recursive and odd even bubble sorts.
- `circle_sort_lib`: recursive and bottom up circle sorts.
- `comb_sort_lib`: comb sort over six shrink ratios.
- `cycle_sort_lib`: cycle sort.
- `fun_sorts_lib`: slow, stooge, cyclent, quick surrender, random shell, bad heap.
- `heap_sort_lib`: the heap machinery and the classic, quick build and dual build heap sorts.
- `index_heap_sort_lib`: index heap sort, a heap of indices keyed by element then index, stable at one word per element.
- `insertion_sort_lib`: insertion sort over a strategy.
- `merge_sort_lib`: classic and rotation merge sorts, the merge kinds.
- `quick_heap_sort_lib`: the quick heap hybrid and the set quick select family.
- `quick_select_lib`: recursive and iterative quick select.
- `quick_sort_lib`: the quick sort drivers and the partition composites.
- `rod_sort_lib`: rod sort over a branching strategy and a merge.
- `shell_sort_lib`: shell sorts over nine gap sequences.
- `tagged_heap_sort_lib`: tagged heap sort, a stable heap sort in pre-order with a stamp only for the element a pop moves.
- `tree_sort_lib`: tree sort over a tree axis, the array as node store, a node in one to four words by length.
- `weak_heap_sort_lib`: weak heap sort over a direction and a bit storage.
