# `tree_sort_lib`

Tree sort over a tree axis. The array is the node store of a binary search
tree: a side array holds each node's parent, left and right index and a rank,
a `Layout` picked by the length of the array. A node takes one word up to 2^19
elements, two words up to 2^32 and four beyond, so no length is too long for
the store and the sort has no input cap. Every element is attached as a leaf
by key, then the leftmost node is deleted again and again, the node occupying
the next output slot moves into the freed slot, and the minimum is written
there, so the sorted prefix grows from the left the way heap sort's sorted
suffix grows from the right. The tree decides what happens to the shape after
each attach and each deletion: `avl` retraces and rotates, `naive` does
nothing. One to four words of extra space per element, by length, stable.

Its catalog components are declared in `tree.spec`.

Query `tree.avbs` builds: `TreeSorts`.
