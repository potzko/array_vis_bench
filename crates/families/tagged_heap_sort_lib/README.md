# `tagged_heap_sort_lib`

Tagged heap sort, `TaggedHeapSort`: a stable heap sort whose extra data is a
tag per element that a pop has moved, rather than a word per element.

A heap sort is unstable because its array holds nothing about an element but
its position, and a pop moves the element at the array's end to the root,
across every equal of it. This sort keeps the order of equal elements as
their positions in a pre-order layout of the complete binary tree, and
arranges that no operation but that one move ever passes an equal: the heap
is built by selection rather than by sifting, since a sinking element could
pass an equal in the left subtree, and in a sift down a tie between children
goes left, which is an adjacent cell, while the right child rises over the
left subtree only when strictly smaller or when the left child is a tagged
equal, whose subtree holds no untagged equal. The one element that does cross
the heap, the jumper, is tagged with a stamp at the moment it jumps, when its
order is still known: it was the last of its untagged equals, and among
jumpers of one value the newer is the earlier. A min heap, pops written to the
end, and one reversal at the finish leave equals in their original order.

A cell is held as a stamped region for as long as its stamp is in the map,
so the visualiser paints the jumpers.

The build is `O(n log n)` comparisons and moves, the pops `O(n log n)`, and
the tags live in a map from cell to stamp that holds the jumpers still in the
heap: a fraction of `n` on random data, all of it in the worst case, so the
space is `O(n)` and the question the catalog answers is whether the map's
updates cost more than the word per element they save.

Its catalog component is declared in `tagged_heap.spec` and built by
`tagged_heap.avbs`.
