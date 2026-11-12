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

## Verdict

- The index heap is the stable heap sort: about twice the unstable heap at
  2^18 and three times at 2^20, where the indirection stops fitting in cache,
  level with the bufferless block sort at 2^18 and behind it at 2^20.
- The level sort was stable and in place at about twice the bufferless block
  sort, and a quicksort by another name, so it did not stay.
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
