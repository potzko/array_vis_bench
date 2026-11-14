# `smooth_sort_lib`

Smooth sort, `SmoothSort<D, S, F, R, B>`: a heap sort in place whose heap is a
forest of trees laid out in post order, grown one element at a time. With the
roots kept ascending the greatest element is always the last, so taking it off
is the whole of the swap to the end, what is left to do is to set the two
subtrees of the tree it was the root of among the roots, and a sorted input
costs a few comparisons an element and no moves, where a binary heap sort pays
its logarithm all the same.

The sort is a `HeapAlgorithm` of `heap_sort_lib`, so the loop that builds and
then extracts is the one every heap sort here runs. Its forest is the state
`HeapAlgorithm` carries, a list of the orders of its trees, a logarithm of
words, and the root of the whole heap that `root_phys` names is the last cell.

Five axes make the 72 entries:

- `D`, the heap direction, `HeapDir` of `heap.spec`, shared with the other heap
  sorts: a max heap from the left or a min heap from the right. Every position
  the sort holds is logical, and the direction maps it to a cell.
- `S`, the shape of the trees, `Shape`, which says how big a tree is, what its
  subtrees are and when the next element joins the last two trees. `Leonardo`
  is Dijkstra's, a root over a tree of the order before and one of the order
  before that. `Perfect` is the poplar sort's, a root over two trees of the
  order before, a forest the count in skew binary. `Narayana` is a root over
  trees of the order before and of three before, which grow by 1.47 an order
  instead of 1.62, so a tree is deeper for its size and its right subtree is
  the small one.
- `F`, the sift of one tree, `Sift`. `Pairs` takes the greater child at each
  level against the root, two comparisons a level and a swap. `BottomUp` is
  Floyd's: the greater child all the way to a leaf, one comparison a level, and
  a climb from the leaf to where the root belongs, with a write a level in
  place of a swap. `BinarySearch` finds that place by a search of the path, as
  the path descends.
- `R`, what is done about the order of the roots, `Roots`. `SortedRoots` puts
  each root among the roots as it joins, so that the greatest is last.
  `ScannedRoots` leaves them as they come and finds the greatest at each
  extraction, a comparison for each root, as the poplar sort does, and trades
  it with the last element.
- `B`, the build of the forest, `Build`. `Insert` adds the elements one at a
  time and settles each as it comes, `LazyInsert` leaves out that step for a
  tree the next element is about to join, which makes it a child whose place
  among the roots is of no account, and `Heapify` lays the whole forest down
  and makes each tree a heap from the leaves up, before one pass puts the roots
  in order. With the roots scanned nothing is ever put in order, a lazy
  insertion has nothing to leave out and a heapify does what an insertion does,
  so the query keeps `Insert` alone.

Putting a root among the sorted roots is the one operation the two phases
share: a root that outranks the last one trades places with it, and the tree
that took the smaller element is sifted once where the chain of trades stopped.
A tree after the first in the chain holds the children of the root that left
it, so the element coming in must outrank those too or it stays where it is.

## Cost

Comparisons and moves an element, a swap or a write counting as one move, on a
random input of 16384 elements, and the comparisons an element of an input
already sorted. The min reverse direction costs the same to a hundredth.

| trees | sift | roots | build | random comparisons | random moves | sorted comparisons |
|---|---|---|---|---|---|---|
| leonardo | pairs | sorted | insert | 39.7 | 17.5 | 3.00 |
| leonardo | pairs | sorted | lazy insert | 37.4 | 16.5 | 2.50 |
| leonardo | pairs | sorted | heapify | 34.6 | 15.4 | 2.00 |
| leonardo | pairs | scanned | insert | 28.0 | 12.0 | 5.63 |
| leonardo | bottom up | sorted | insert | 31.1 | 19.5 | 3.61 |
| leonardo | bottom up | sorted | lazy insert | 28.9 | 18.2 | 3.11 |
| leonardo | bottom up | sorted | heapify | 26.6 | 16.8 | 2.61 |
| leonardo | bottom up | scanned | insert | 19.9 | 13.4 | 6.25 |
| leonardo | binary search | sorted | insert | 33.2 | 19.5 | 3.38 |
| leonardo | binary search | sorted | lazy insert | 31.0 | 18.2 | 2.88 |
| leonardo | binary search | sorted | heapify | 28.4 | 16.8 | 2.38 |
| leonardo | binary search | scanned | insert | 21.7 | 13.4 | 6.01 |
| perfect | pairs | sorted | insert | 42.6 | 18.2 | 3.00 |
| perfect | pairs | sorted | lazy insert | 40.4 | 17.2 | 2.50 |
| perfect | pairs | sorted | heapify | 36.6 | 15.7 | 2.00 |
| perfect | pairs | scanned | insert | 27.6 | 11.1 | 7.00 |
| perfect | bottom up | sorted | insert | 34.2 | 20.2 | 4.00 |
| perfect | bottom up | sorted | lazy insert | 32.2 | 19.0 | 3.50 |
| perfect | bottom up | sorted | heapify | 29.2 | 17.1 | 3.00 |
| perfect | bottom up | scanned | insert | 20.1 | 12.5 | 8.00 |
| perfect | binary search | sorted | insert | 36.6 | 20.2 | 3.63 |
| perfect | binary search | sorted | lazy insert | 34.4 | 19.0 | 3.13 |
| perfect | binary search | sorted | heapify | 31.1 | 17.1 | 2.63 |
| perfect | binary search | scanned | insert | 22.0 | 12.5 | 7.63 |
| narayana | pairs | sorted | insert | 39.8 | 17.7 | 3.00 |
| narayana | pairs | sorted | lazy insert | 37.4 | 16.7 | 2.50 |
| narayana | pairs | sorted | heapify | 35.0 | 15.8 | 2.00 |
| narayana | pairs | scanned | insert | 29.2 | 12.8 | 5.13 |
| narayana | bottom up | sorted | insert | 30.6 | 19.7 | 3.46 |
| narayana | bottom up | sorted | lazy insert | 28.4 | 18.4 | 2.96 |
| narayana | bottom up | sorted | heapify | 26.4 | 17.2 | 2.46 |
| narayana | bottom up | scanned | insert | 20.6 | 14.2 | 5.59 |
| narayana | binary search | sorted | insert | 32.7 | 19.7 | 3.28 |
| narayana | binary search | sorted | lazy insert | 30.4 | 18.4 | 2.78 |
| narayana | binary search | sorted | heapify | 28.2 | 17.2 | 2.28 |
| narayana | binary search | scanned | insert | 22.4 | 14.2 | 5.41 |

The binary heap sort of the catalog, built by the iterative heapify, spends
24.9 comparisons and 13.1 moves an element on the same random input and 25.7
comparisons on the sorted one.

- With the roots sorted the smooth sorts cost more on random input: 26.4
  comparisons at best, the Leonardo or Narayana trees sifted from the bottom and
  built by heapify, 6 percent over the binary heap sort, and 42.6 at worst. On
  sorted input every one of them is a sixth of the binary heap sort or less, 2
  to 4 comparisons, and moves nothing.
- With the roots scanned the cost on random input falls, to 19.9 comparisons and
  13.4 moves for the Leonardo trees sifted from the bottom, a fifth under the
  binary heap sort in comparisons, and the sorted input is no longer cheap, 5 to
  8 comparisons an element for the scan of the roots, though it still moves
  nothing. Not ordering the roots as they join saves more than the scan costs.
- The binary search of the path costs about two comparisons an element more than
  the climb it replaces, with the same moves: the element a sift carries is
  nearly always one that belongs by the leaves, so the climb stops after a step
  or two and a search of the whole path cannot.
- The Leonardo and Narayana trees cost the same to a few percent, the perfect
  trees more, with more trees in the forest to walk.

The worst case is `O(n log n)`, the best `O(n)` with the roots sorted and `O(n
log n)` with them scanned, and the space a logarithm of words for the orders of
the forest.

Its catalog components are declared in `smooth.spec` and built by `smooth.avbs`.
