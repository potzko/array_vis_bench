# `avl_sort_lib`

AVL sort, a single driver with no axes. The array is the node store of an AVL
tree: a side array holds one packed word per node with its parent, left and
right index and its height. The tree is built by insertion, then the leftmost
node is deleted again and again, the node occupying the next output slot moves
into the freed slot, and the minimum is written there, so the sorted prefix
grows from the left the way heap sort's sorted suffix grows from the right.
O(N log N) in every case, one word of extra space per element, stable.

Its catalog component is declared in `avl.spec`.

Query `avl.avbs` builds: `AvlSorts`.
