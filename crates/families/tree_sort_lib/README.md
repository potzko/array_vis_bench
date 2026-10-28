# `tree_sort_lib`

Tree sort over a tree axis. The array is the node store of a binary search
tree: a side array holds one packed word per node with its parent, left and
right index and a rank. Every element is attached as a leaf by key, then the
leftmost node is deleted again and again, the node occupying the next output
slot moves into the freed slot, and the minimum is written there, so the
sorted prefix grows from the left the way heap sort's sorted suffix grows
from the right. The tree decides what happens to the shape after each attach
and each deletion: `avl` retraces and rotates, `naive` does nothing. One word
of extra space per element, stable.

Its catalog components are declared in `tree.spec`.

Query `tree.avbs` builds: `TreeSorts`.
