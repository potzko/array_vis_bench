# `small_sorts`

The small sort axis: strategies for the base case of a recursive sort, and
the sorting networks. Declared under several roles so quick sort, merge sort,
the fun sorts and the first class small sort kind each see the population
they want.

- `small_sort_basic`: no small sort, size one, size two.
- `small_sort_insertion`: insertion sort up to a threshold.
- `small_sort_insertion_strategy`: linear and binary insertion.
- `small_sort_deferred_insertion`: one final insertion pass after a deferred quicksort.
- `small_sort_network`: the eight element network.
- `small_sort_network_16`: Batcher's sixteen element network.
