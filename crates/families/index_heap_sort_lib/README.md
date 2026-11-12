# `index_heap_sort_lib`

Index heap sort, `IndexHeapSort`: a heap sort that is stable at the price of one
word per element. The heap holds indices into the array, ordered by the element
at each index and then by the index itself, so two equal elements are distinct
to the heap and leave it in index order, which is their original order. Heap
sort is unstable only because it cannot tell equal elements apart, and the
index is the smallest thing that lets it. The heap is built bottom up and
popped `n` times, which leaves the stable sorting permutation in the word
store, and the permutation is applied to the array by following its cycles, so
an element is written once at most and the elements themselves are never
swapped. `n log n` comparisons on every input, `n log n` word writes and at
most `n` element writes, `n` words of extra space, stable.

Its catalog component is declared in `index_heap.spec`.

Query `index_heap.avbs` builds: `IndexHeapSorts`.
