# Implementation plan: the extraction axis

Status: done 2026-11-12, kept as the record. The question was whether a block
merge sort carves its keys and buffer faster by a stable partition than by the
scan it inherited from GrailSort and WikiSort. It does not, see the numbers;
the sorted prefix is the one extraction that saves anything.

## The question

The scan gathers the first occurrence of each distinct value in a block that
rides behind it, `O(n + c^2)` moves for `c` carved elements, about one move per
element for a square root sized carve. Two alternatives were asked for: sort a
prefix of the array first so that the scan sweeps it without a move, and select
the carved elements by value with a stable partition around a pivot aimed at
some overshoot of the count wanted, retrying until the selection lands within a
window, then sort the selection. The second has a structural upside: keys that
are the smallest values slide back in at the end of the sort without a move,
and that slide is the largest of the carving's costs, three moves per element.

## What was built

Two roles in `array_vis_bench_traits` and their fillers in `extraction_lib`.

- `StablePartition`: the elements below a pivot brought to the front, each
  side keeping its order. `RotationPartition` halves the region, partitions
  each half and rotates the middle, in one buffered pass when the rotation's
  scratch holds the region: one comparison per element, `O(n log n)` moves,
  `O(n log (n / s))` with `s` cells of scratch. `SlidingPartition` rides a block
  of matches behind the scan, `O(n + m r)` moves for `m` matches in `r` runs.
- `Extraction`: `ScanExtraction`, `PrefixSortExtraction` and
  `PartitionExtraction<P>`. The partition extraction sorts the prefix of `want`
  and a tenth, reads the pivot off it at the target's rank, partitions the rest
  by it, and moves the pivot along the sample when the selection falls outside
  `[want, 2 x target]`, at most three rounds and at most a doubling of the rank
  per round. The sample's elements at and above the pivot step over the
  gathered part, which is sorted and merged into the sample's part below, and
  the scan compacts the first occurrences.
- `BlockMergeSort<B, K, R, E>` takes the extraction as its fourth axis. The
  catalog keeps its 390 entries on the scan and adds `BlockMergeExtractions`,
  the three other extractions over the WikiSort, dynamic GrailSort and
  bufferless shapes with in array keys by the reversal, trinity and auxiliary
  rotations, 27 entries.

Stability holds for every filler by one argument: the prefix is the array's
front, so after a stable sort the first of each run of equals is still the
array's first occurrence; a stable partition keeps each side's order; and the
elements left behind only have to keep their order among equals, which is all
the merges ask of them.

## Measured

Logged moves and comparisons per element, and the best of five timed runs
without the logger, on random `usize` keys. "Distinct" draws from eight times
`n` values, "repeats" from `n / 64`.

| shape | extraction | distinct moves | distinct ms | repeats moves | repeats ms |
|---|---|---|---|---|---|
| wiki, reversal, 2^20 | scan | 43.1n | 85.2 | 43.3n | 74.7 |
| | prefix sort | 42.1n | 84.7 | 42.3n | 77.2 |
| | partition by rotation | 45.1n | 91.0 | 48.2n | 82.1 |
| | partition by sliding | 43.8n | 86.4 | 48.2n | 79.3 |
| wiki, auxiliary, 2^20 | scan | 43.8n | 84.8 | 44.1n | 77.2 |
| | prefix sort | 42.8n | 84.5 | 43.1n | 75.6 |
| | partition by rotation | 41.0n | 86.5 | 44.6n | 75.3 |
| grail sqrt, reversal, 2^20 | scan | 40.6n | 82.0 | 40.6n | 73.5 |
| | prefix sort | 40.3n | 82.7 | 40.3n | 72.6 |
| | partition by rotation | 48.1n | 94.4 | 46.2n | 79.8 |
| | partition by sliding | 43.8n | 83.3 | 46.0n | 75.4 |

Comparisons are the same to within a third of a percent for the prefix sort
and one per element more for the partitions, which is the partition itself.

## Verdict

- The prefix sort saves about one move per element, a fortieth of the sort,
  and no time anyone can measure. It is the extraction to prefer, and it is
  what the earlier analysis predicted: the extraction was never more than a
  fortieth of the sort to begin with.
- The partition's free slide back is real on distinct data, three moves per
  element gone, but the partition costs more than that on the in place shapes:
  the rotation partition about six moves per element without scratch, and the
  sort is six to nine percent slower. With the auxiliary rotation's half array
  of scratch the partition is one pass and the sort moves seven percent fewer
  elements, but those shapes are not in place, and the time is within noise.
- With repeats the slide back is not free, since the duplicates of the carved
  values sit between the keys, and the partition is a pure loss.
- The sliding partition is quadratic in what it gathers. Before the retry's
  rank was capped at a doubling, one unlucky sample at 2^20 gathered ten
  times the target and the sort took half again as long.

The carving's remaining quadratic steps, the slide back and the insertion sort
of the buffer, are the same for every extraction and are where the moves are:
three per element and a third, against the extraction's one. Any further gain
is there, not in the extraction.
