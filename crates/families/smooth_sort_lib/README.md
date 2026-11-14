# `smooth_sort_lib`

Smooth sort, a heap sort in place whose heap is a forest of trees laid out in
post order, grown one element at a time. The greatest element is always the
last, so a sorted input costs a linear number of comparisons.
