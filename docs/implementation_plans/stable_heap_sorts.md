# Implementation plan: stable heap sorts

Status: done 2026-11-12, kept as the record. Two stable heap sorts were built
to settle whether a heap can sort stably at less than a word per element: the
index heap, which pays the word, and the tagged heap, which pays a map entry
per element a pop has moved. The index heap wins; the map costs more than the
word it saves.

## The obstacle

A heap stores nothing about an element but its position, and a pop moves the
element at the array's end to the root, across every equal of it. Two equal
elements never have to be compared for their order to be lost. Any stable
heap sort therefore has to carry the order of equals somewhere, and the only
question is where and for whom.

## The two sorts

- `IndexHeapSort`, in `index_heap_sort_lib`: a heap of indices keyed by the
  element and then by the index, so equal elements are distinct to the heap
  and pop in index order. The pops leave the stable permutation in the word
  store, applied to the array by following its cycles, so each element is
  written once. One word per element, `O(n log n)` comparisons, one
  comparison per heap step through one indirection.
- `TaggedHeapSort`, in `tagged_heap_sort_lib`: the heap in the pre-order
  layout of the complete binary tree, built by selection rather than by
  sifting, since a sinking untagged element could pass an equal in the left
  subtree. Untagged equals keep their order as their cells. A tie between
  children goes left, an adjacent cell; the right child rises over the left
  subtree only when strictly smaller or when the left child is a tagged
  equal, whose subtree holds no untagged equal. The one element a pop moves
  from the end to the root is tagged with a stamp while its order is still
  known: it was the last of its untagged equals, and among jumpers of one
  value the newer is the earlier. The root is then always the pop target,
  which the invariants prove and the tests check after every pop. A min
  heap, pops to the end and one reversal leave equals in original order.
  The map, keyed by cell, hashes by one multiplication, and only a tagged
  rising child touches it during a sift.

## Measured

Best of five timed runs without the logger, logged moves and comparisons per
element, random `usize` keys below eight times `n` (distinct) and below
`n / 64` (repeats), on a quiet machine.

| sort | 2^18 distinct | 2^18 repeats | 2^20 distinct | 2^20 repeats |
|---|---|---|---|---|
| classic binary heap, unstable | 19.5 ms | 19.3 ms | 97 ms | 95 ms |
| index heap | 41.1 ms | 40.7 ms | 281 ms | 284 ms |
| tagged heap | 71.7 ms | 76.1 ms | 416 ms | 442 ms |
| bufferless block sort, stable in place | 41.9 ms | 37.6 ms | 200 ms | 181 ms |
| standard library stable sort | 3.4 ms | 1.7 ms | 17 ms | 12 ms |

At 2^18 the index heap moves 1.0n elements and compares 41.6n times, the
tagged heap 25.7n and 56.3n, the classic heap 17.1n and 32.9n. The tagged
heap's map peaks at 0.25n entries on random data, since a jumper cannot know
it has no equals and stays tagged until it is popped itself.

## The level sort, built and removed

The lazy refinement of the level ordered heap was built as `level_sort_lib`
and removed the same day, since its extraction is a quicksort and the heap
is only the name of what its build leaves behind; the numbers stay as the
record of what that route costs. Stable selection at the power of two
boundaries from the largest down, then each level halved by stable selection
of its median down to insertion sorted runs. A stable selection is a quickselect over the three
way stable partition, a median of three compared in place as the pivot,
stopping once the wanted boundary falls inside the pivot's equal group. No
element ever passes an equal, so it is stable with nothing but the
rotation's scratch, and the all equal input costs comparisons but no moves.
It is a stable quicksort with exact median splits by another route.

| sort | 2^18 distinct | 2^18 repeats | 2^20 distinct | 2^20 repeats |
|---|---|---|---|---|
| level sort, reversal | 86 ms | 70 ms | 391 ms | 338 ms |
| bufferless block sort, reversal | 42 ms | 37 ms | 198 ms | 178 ms |

At 2^18 it moves 92n elements and compares 55n times against the block
sort's 70n and 24n: the exact medians cost the selection's constant, about
three and a half passes over a range per split, on every level. It is the
one stable sort of the three that is in place, and the cheaper form of it is
plain: one stable three way partition per level around a sampled pivot, no
selection, which halves the comparisons at the price of unbalanced splits.
That is the rotation partition as a `PartitionScheme` filler for the quick
sort drivers, still unbuilt. The three way split the selection needed stays
in the partition role.

## The level heap sort

The level ordered heap used as it should be, in `level_heap_sort_lib`,
built after the level sort was removed. The jump is the only unsafe move of
a heap, and out of place it is not needed: a pop takes the root and the hole
sinks instead, the smaller live child rising, ties to the left, until it
dies at a leaf, marked by one byte. In the pre-order layout both rising
moves keep equals in order, and with the selection build so does the
build, so the pops are stable with no stamp and no index; the price is a
buffer to hold the heap and a mark per buffer cell. The levels are what
make the buffer smaller than the array: a level's sorted output is its own
cell range, so each level is swapped into the buffer, heaped and popped
back, each dead leaf taking the buffer element the output cell held, and a
level larger than the buffer is levelled again, the ranges halving with
depth. A carved buffer is distinct and goes home as a block sort's does.
`LevelHeapSort<B, P, R>` over the block buffers, the rotation partition and
the reversal rotation, twelve entries under heap sorts.

Best of five, random `usize` keys below eight times `n`:

| sort | 2^18 | 2^20 | extra space |
|---|---|---|---|
| level heap, external half | 46 ms | 202 ms | n / 2 words, n / 2 bytes |
| level heap, external sqrt | 86 ms | 385 ms | sqrt n words and bytes |
| level heap, in array sqrt | 83 ms | 377 ms | sqrt n bytes |
| level heap, in array log | 99 ms | 468 ms | log n bytes |
| index heap | 42 ms | 274 ms | n words |
| tagged heap | 80 ms | 447 ms | a map, n / 4 entries at peak |
| bufferless block sort | 41 ms | 194 ms | none |
| wiki block sort, in array sqrt | 18 ms | 81 ms | none |

The half buffer matches the index heap at 2^18 and beats it at 2^20 with
half the words. The square root shapes move about a hundred elements per
element, two and a half times the half buffer's, and that is where the
small buffers pay: the level build selects by stable partitions that rotate
with no scratch, a depth of log of `n` over the buffer, and every depth
partitions the whole range. The heaps themselves are a small part of the
time. The in array half and sixteenth carve their buffer by the scan at the
square, as the block sort's do, and are quadratic.

Two cuts are open. The external buffer is free while a level is being
built and could serve the partition as scratch, which makes a partition of
a range at most twice the buffer one buffered pass; for a carved buffer the
pass has to swap rather than write, since the buffer's elements are real.
And the selection build of each level's heap is `m log m` comparisons where
a heapify is linear, the same cost the tagged heap carried.

### The marks, an axis

The mark per buffer cell was a byte of an aux array, the one allocation of
a shape with a carved buffer. It is the `Marks` slot of
`LevelHeapSort<B, M, P, R>`, three fillers in `scratchspace_lib`: the byte,
a bit packed into words, and two more distinct elements of the array per
cell, carved in the same run as the buffer, in order while the cell is live
and swapped once it is dead. A carved mark is read by a comparison and
written by a swap, each pop ends by clearing the marks of its level, which
are all set by then, so no write reads first, and the pairs are sorted again
whenever a level is done. With a carved buffer and carved marks the sort
allocates nothing. A shape that finds fewer distinct elements than it asks
for has the cells they pay for, a third of them each with carved marks.

Best of five, the same keys, comparisons and moves per element at 2^20,
measured together on another day than the table above:

| sort | 2^18 | 2^20 | comparisons | moves | extra space |
|---|---|---|---|---|---|
| in array 10 log, bytes | 78 ms | 342 ms | 70.2 | 123.9 | 10 log n bytes |
| in array 10 log, bits | 76 ms | 335 ms | 70.2 | 123.9 | 10 log n bits, at most ten words |
| in array 10 log, in array | 70 ms | 331 ms | 79.6 | 123.1 | none |
| in array sqrt, bytes | 64 ms | 291 ms | 64.1 | 128.2 | sqrt n bytes |
| in array sqrt, bits | 64 ms | 292 ms | 64.1 | 128.2 | sqrt n bits |
| in array sqrt, in array | 64 ms | 283 ms | 77.0 | 126.1 | none |
| external half, bytes | 36 ms | 169 ms | 40.6 | 68.4 | n / 2 words, n / 2 bytes |
| external half, bits | 35 ms | 167 ms | 40.6 | 68.4 | n / 2 words, n / 2 bits |

The carved marks cost a comparison per child the sinking hole looks at,
about a seventh more comparisons over the sort, and no time on integer
keys: the heaps are a small part of the work and the marks sit in the
array beside them. The bits cost nothing over the bytes.

### The time, counted in moves

The bounds first declared for the family counted comparisons. In moves a
stable partition with no scratch is `n log n`, and the level build runs
one over every element at each depth, so a shape over an in place
rotation with a buffer smaller than linear is `O(n log^2 n)` expected and
`O(n^2 log n)` at worst, the quickselect taking a pass per element. Two
things take the log back: a linear buffer, reached in a constant number of
depths, and a rotation with linear scratch, which the partition makes one
pass through. Writes per `n log n` on random keys, a swap as two:

| sort | 2^12 | 2^14 | 2^16 | 2^18 | 2^20 |
|---|---|---|---|---|---|
| in array sqrt, reversal | 7.3 | 8.2 | 9.4 | 10.8 | 11.4 |
| in array 10 log, reversal | 7.1 | 8.6 | 9.4 | 10.3 | 11.9 |
| external half, reversal | 4.9 | 5.5 | 5.7 | 5.4 | 6.0 |
| in array sqrt, auxiliary | 3.3 | 3.2 | 3.3 | 3.4 | 3.3 |
| in array 10 log, auxiliary | 3.4 | 3.4 | 3.4 | 3.5 | 3.5 |

The comparisons per `n log n` stay between 2 and 3.5 in every row.

## Verdict

- The index heap is the stable heap sort: about twice the unstable heap at
  2^18 and three times at 2^20, where the indirection stops fitting in cache,
  level with the bufferless block sort at 2^18 and behind it at 2^20.
- The level sort was stable and in place at about twice the bufferless block
  sort, and a quicksort by another name, so it did not stay.
- The level heap sort is the heap sort the level property was for: stable
  with a buffer of any size, half the index heap's words at its speed, and a
  curve down to log n bytes at about twice the time, or to no allocation at
  all with the marks carved from the array.
- The tagged heap works, and every move but the jump is order safe, but it
  costs about four times the unstable heap. Its comparisons are higher
  because the selection build is `n log n` where a heapify is `2n`, and a
  comparison on a tie needs a second one; its time is the map, one hash
  lookup per sift step for the rising child and the entries that ride with
  tagged cells. The map holds a quarter of the elements at peak, which at
  two words an entry and the table's load is most of the word per element it
  was meant to save.
- A bit set of tagged cells would remove the lookup for untagged children
  and cut the map to tagged versus tagged ties, at `n` bits of fixed space.
  That is the next cut if the tagged heap is ever worth more time; it would
  not change which sort wins.
