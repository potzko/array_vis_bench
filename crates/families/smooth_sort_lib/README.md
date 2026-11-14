# `smooth_sort_lib`

Smooth sort, `SmoothSort<D, S, F, B>`: a heap sort in place whose heap is a
forest of trees laid out in post order, each tree a heap and the roots
ascending, so that the greatest element is always the last. Taking it off is
the whole of the swap to the end, and what is left to do is to set the two
subtrees of the tree it was the root of among the roots. A sorted input costs a
few comparisons an element and no moves, where a binary heap sort pays its
logarithm all the same.

The sort is a `HeapAlgorithm` of `heap_sort_lib`, so the loop that builds and
then extracts is the one every heap sort here runs. Its forest is the state
`HeapAlgorithm` carries, a list of the orders of its trees, a logarithm of
words, and the root of the whole heap that `root_phys` names is the last cell.

Four axes make the 24 entries:

- `D`, the heap direction, `HeapDir` of `heap.spec`, shared with the other heap
  sorts: a max heap from the left or a min heap from the right. Every position
  the sort holds is logical, and the direction maps it to a cell.
- `S`, the shape of the trees, `Shape`: `Leonardo`, a root over a tree of the
  order before and one of the order before that, Dijkstra's, or `Perfect`, a root over
  two trees of the order before, the poplar sort's. The shape says how big a
  tree is, what its subtrees are and when the next element joins the last two
  trees. The Leonardo forest is the count in a few trees, the perfect forest
  the count in skew binary, and a Leonardo tree sifts through uneven subtrees.
- `F`, the sift of one tree, `Sift`: `Pairs`, the greater child at each level
  against the root, two comparisons a level and a swap, or `BottomUp`, Floyd's,
  the greater child all the way to a leaf, one comparison a level, and a climb
  from the leaf to where the root belongs, with a write a level in place of a
  swap.
- `B`, the build of the forest, `Build`: `Insert` adds the elements one at a
  time and puts each root among the roots as it comes, `LazyInsert` leaves out
  that step for a tree the next element is about to join, which makes it a
  child whose place among the roots is of no account, and `Heapify` lays the
  whole forest down and makes each tree a heap from the leaves up, before one
  pass puts the roots in order.

Putting a root among the roots, `rectify`, is the one operation the two phases
share: a root that outranks the last one trades places with it, and the tree
that took the smaller element is sifted once where the chain of trades stopped.
A tree after the first in the chain holds the children of the root that left
it, so the element coming in must outrank those too or it stays where it is.

## Cost

Comparisons and moves an element, a swap or a write counting as one move, on a
random input of 16384 elements, and the comparisons an element of an input
already sorted. The min reverse direction costs the same to a hundredth.

| trees | sift | build | random comparisons | random moves | sorted comparisons |
|---|---|---|---|---|---|
| leonardo | pairs | insert | 39.8 | 17.5 | 3.00 |
| leonardo | pairs | lazy insert | 37.4 | 16.5 | 2.50 |
| leonardo | pairs | heapify | 34.7 | 15.4 | 2.00 |
| leonardo | bottom up | insert | 31.1 | 19.5 | 3.61 |
| leonardo | bottom up | lazy insert | 28.9 | 18.2 | 3.11 |
| leonardo | bottom up | heapify | 26.6 | 16.8 | 2.61 |
| perfect | pairs | insert | 42.6 | 18.2 | 3.00 |
| perfect | pairs | lazy insert | 40.4 | 17.2 | 2.50 |
| perfect | pairs | heapify | 36.6 | 15.7 | 2.00 |
| perfect | bottom up | insert | 34.2 | 20.2 | 4.00 |
| perfect | bottom up | lazy insert | 32.3 | 19.0 | 3.50 |
| perfect | bottom up | heapify | 29.2 | 17.1 | 3.00 |

The binary heap sort of the catalog, built by the iterative heapify, spends
24.9 comparisons and 13.1 moves an element on the same random input and 25.7
comparisons on the sorted one. On random input the smooth sorts cost more: the
best, the Leonardo trees sifted from the bottom and built by heapify, 7 percent
more comparisons and 28 percent more moves, the worst 71 percent more
comparisons. On sorted input every one of them is a sixth of the binary heap
sort or less, and moves nothing. The build moves the comparisons most, the sift
trades comparisons against moves, and the shape is the smallest of the three.

The worst case is `O(n log n)`, the best `O(n)`, and the space a logarithm of
words for the orders of the forest.

Its catalog components are declared in `smooth.spec` and built by `smooth.avbs`.
