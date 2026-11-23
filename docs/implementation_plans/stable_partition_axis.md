# Implementation plan: the stable partition axis

Status: done 2026-11-23, kept as the record. The question was whether a stable
partition can run in linear time without rotating, and what a partition like
that buys the sorts that partition. It can: `EncodedPartition` moves each
element about three times and compares it about once, in `O(log n)` words, and
nothing at all when there is nothing to move. It makes a stable quick sort as
fast as the wiki block sort, makes the level heap sort about three times faster
on its small buffers, and changes nothing for the block merge sort, whose
extraction was never the cost.

## The question

The `StablePartition` role had two fillers. `RotationPartition` halves the
region and rotates the middle, `O(n log n)` moves, `O(n log (n / s))` with `s`
cells of scratch. `SlidingPartition` rides a block of matches behind the scan,
`O(n + m r)` moves, which on a split of random keys is `n^2 / 16`. Both are
dear: the plain stable quick sort that the stable heap sorts record left
unbuilt, one stable three way partition a level around a sampled pivot, had only
these two to stand on.

Katajainen and Pasanen, "Stable minimum space partitioning in linear time"
(BIT, 1992), and aphitorite's Logsort partition do it in linear time with
`O(log n)` cells of space. This is a clean room version of the idea, written
from its outline.

## What was built

- The role. `StablePartition` and `Extraction` gained `scratch_size(n)`,
  default 0: the cells a filler wants for itself on top of the rotation's. A
  sort allocates the larger of the two once, through the logger, as it did for
  the rotation alone. `LevelHeapSort` takes `P::scratch_size`, `BlockMergeSort`
  takes `E::scratch_size` and `PartitionExtraction<P>` reports `P`'s, and the
  new sort takes `P`'s. One scratch is enough because the encoded partition
  lets go of its cells before its last rotation uses them.
- `EncodedPartition`, in `extraction_lib`, label `encoded`, `MOVES = N1`. It
  leaves the front elements a region starts with and the back elements it ends
  with where they are, so a pass with nothing to move costs its comparisons.
- `StableQuickSort<P, R>`, in `stable_quick_sort_lib`, under `quick sorts /
  stable`. The pivot is a copied value, the median of three and past 128
  elements the median of three medians of three, so nothing is moved to choose
  it. The split is one `partition`, below the pivot and the rest, each in
  order, and a second pass, `partition_at_most`, only when the first left
  nothing below the pivot, which peels the equals off a region that would
  otherwise not shrink; the sort recurses into the smaller of the two parts and
  loops on the larger, so the stack is `log n` deep, and a region of 16 or
  fewer is one insertion sort, `sort_region` of the traits crate. The worst
  case is quadratic on adversarial pivots and the bounds say so: `n` levels of
  the partition's moves, `n^2` with the encoded partition.
- Catalog. 30 entries, the rotation and encoded partitions by every rotation.
  The sliding partition is left out: on half matching input it costs `n^2 / 16`
  moves a pass, 5.7 s for the sort at 2^18 elements and 93 s at 2^20, and a
  component takes one `max_input`, not one per filler. Two small studies
  ride on the new filler: the `BlockMergeExtractions` study gains the partition
  extraction over `encoded` for its three shapes and three rotations, 9
  entries, and the level heap sort gains `LevelHeapEncoded`, the encoded
  partition over every buffer by the reversal and auxiliary rotations, 28
  entries. They are the last two commits of the branch, and can be dropped
  without touching the rest. The catalog goes from 2467 entries to 2534, 2497
  without the studies.

## Why it is stable, and why it is linear

Call the elements that go to the front, below the pivot or not above it,
front, and the rest back. Every test of an element's type is one logged
comparison against the pivot. The block length is `b`, eight times the base
two logarithm of the region, rounded up; the scratch is `b` cells. A region the
scratch holds is partitioned by the one pass buffered partition instead.

0. Trim. The front elements the region starts with and the back elements it
   ends with are in place. They are compared once and left, and the rest, which
   starts with a back element and ends with a front one, is what the steps
   below partition. All front, all back and sorted regions end here with no move.
1. Blocks. One scan from the left keeps a write point `w`, below which are
   whole blocks of `b` cells of one type, a count `z < b` of loose front
   elements gathered at `[w, w + z)`, and a buffer of the back elements seen
   since the last block, `B`. The cells from `w + z` to the scan are free, and
   exactly as many as `B` holds, since each element read was either moved down,
   into `B`, or is under the write point. A front element is copied to `w + z`,
   or stays when it is there already; a back element is copied into `B`; when
   `B` is full the `z` loose elements step up over the free cells, which `z < b`
   keeps clear of their old place, and `B` is copied into the block they left.
   No cell of the scratch is read before it is written. Under three moves an
   element: a front element moves once, a back element once into `B` and once
   out, and the loose elements step up once for each whole block of back
   elements, which is under one more move for each of those.
2. Order the blocks. A block's type is the type of its last cell, which
   the steps below never touch. Count the front blocks and the back blocks; if one
   count is zero the blocks are in order. Otherwise the type with fewer blocks
   is the minority, and it is the minority that gets scrambled, so it has to
   carry its rank. Pair the `j`th front block with the `j`th back block for each
   `j` below the minority count, and for each set bit `t` of `j` swap cell `t`
   of one with cell `t` of the other. The two elements were of different types,
   which is the whole trick: after the swap the minority block holds an element
   of the other type at exactly the cells of the bits of its rank, so the pivot
   reads the rank back, and no element is ever equal to the one it swaps with,
   since the pivot told them apart. The rank takes `ceil(log2(minority))` cells,
   fewer than `b`, so the last cell is never one of them. The majority blocks
   then sweep to their end of the row by block swaps, from the left when they
   are front blocks and from the right when they are back blocks, past a gap of
   minority blocks that comes out scrambled; each minority block reads its rank
   and a cycle leader swaps it into place, every swap settling one block; and
   the swaps of the code are undone, which needs no comparison because the
   block at `j` has the rank `j`.
3. Ends. `B` is copied into the free cells behind the loose front elements,
   where its back elements belong, and one rotation of the back blocks past the
   loose elements brings them in front. That is the `Rotation` axis, and the
   scratch is free for it by then.

Stable because no step compares one element with another: each type's elements
leave phase 1 in order, in blocks, the blocks of the majority sweep in order,
the minority blocks come back in order by rank, the code swaps are exchanged
twice and so undone, and the loose and held elements are placed in order. Linear
because phase 1 is three moves an element at most, phase 2 touches each block a
constant number of times (the sweep swaps at most the majority blocks, the cycle
leader at most the minority blocks, and the code is `bits` swaps a pair, `bits
/ b` of a move an element at most) and phase 3 is one rotation of up to `n`
elements.

The block length is the one constant that was chosen by measurement. One
logarithm is enough for the arithmetic, and the tests keep it as the least
block, but a block eight times that cuts the blocks to an eighth and with them
the type reads, the rank reads and the scattered block swaps. One partition of
2^20 random elements half of them in front took 10.8 ms at one logarithm, 7.1
at two, 6.0 at four, 5.2 at eight and 5.2 at sixteen, with 1.9, 1.4, 1.2, 1.1
and 1.1 comparisons an element. The scratch is still `8 log n` words, 160 at
2^20.

The tests run every size to 300 and a few above, fifteen shapes of input (one
type throughout, either type in the minority, alternating blocks and elements,
runs, sorted, reversed, a lone block or element of the other type, blocks in
random order), the three splits, four rotations, a scratch the length asked for
or longer up to the whole region, and blocks of the least length, against a
reference that spells out the order. A replay logger checks that no move goes
unlogged and that no cell of the scratch is read before it is written. Sorted,
all front and all back are pinned at no move and a comparison an element, and a
shuffled middle between two runs at the moves of the middle alone. The
sort's tests pin that distinct keys never make the second pass, that a region
of one value makes it once, and that every key equal, a pivot that is the least
key of its region and a few values with many of each end and stay stable.

## Measured

Best of five timed runs without the logger, logged moves and comparisons per
element, random `usize` keys below eight times `n` (distinct) and below `n / 64`
(repeats), on the development machine with a browser open. A scratch binary over
the worktree's crates, with the repo's profile and flags, not committed. It
reproduces the earlier record's references to within noise: the bufferless block sort 41.6 and 198 ms against 41.9 and 200,
the standard library stable sort 3.9 and 16.3 against 3.4 and 17. A swap counts
as one move. The tables are of the code after both cuts, and the first
measurement, before them, is set beside them in the section on the cuts.

One partition of random keys with half of them in front, the pivot the median
value, so distinct and repeats are one row:

| partition | moves 2^18 | moves 2^20 | comparisons | ms 2^18 | ms 2^20 |
|---|---|---|---|---|---|
| encoded, reversal | 3.27n | 3.28n | 1.10n | 1.20 | 5.05 |
| encoded, auxiliary | 3.27n | 3.28n | 1.10n | 1.21 | 5.03 |
| rotation, reversal | 8.16n | 9.16n | 1.00n | 3.14 | 13.7 |
| rotation, auxiliary, n / 2 words | 2.25n | 2.25n | 1.00n | 0.90 | 3.67 |
| sliding, reversal, 2^16 | 4117n | | 1.00n | 127 | |

The encoded partition is flat, 3.2n moves from 2^12 to 2^20 where the rotation
partition grows with the logarithm. Its constant by shape, at 2^20: the random
half 3.28n moves and 1.10n comparisons; a minority of a sixteenth in front 4.34n
and 1.03n and of a 256th 4.47n and 1.02n, the dearest cases, since every back
block sweeps and rotates past the few front ones; a minority of a sixteenth in
back 2.15n and of a 256th 1.96n; reversed 2.54n, alternating elements 3.52n;
and sorted, all in front and all in back no move at all, one comparison an
element each. Comparisons are 1.0 to 1.1 an element, the type reads of the
blocks and the ranks being a tenth.

The stable quick sort, by the reversal rotation, against the references:

| sort | 2^18 distinct | 2^18 repeats | 2^20 distinct | 2^20 repeats |
|---|---|---|---|---|
| stable quick, encoded | 17.5 ms | 13.3 ms | 81.2 ms | 62.1 ms |
| stable quick, rotation | 46.9 ms | 38.4 ms | 215 ms | 181 ms |
| stable quick, sliding | 5719 ms | 3045 ms | 93490 ms | 73924 ms |
| wiki block sort, in array sqrt | 19.0 ms | 16.9 ms | 87.7 ms | 78.7 ms |
| bufferless block sort | 41.6 ms | 37.6 ms | 198 ms | 177 ms |
| standard library stable sort | 3.9 ms | 2.1 ms | 16.3 ms | 11.4 ms |

The sliding sort is one run each, and its time is the partition's square: the
partition alone is 127 ms at 2^16. Logged moves and comparisons an element:

| sort | 2^18 distinct | 2^18 repeats | 2^20 distinct | 2^20 repeats |
|---|---|---|---|---|
| stable quick, encoded | 45.4n, 21.3n | 41.0n, 16.2n | 53.5n, 23.4n | 46.3n, 18.4n |
| stable quick, rotation | 80.9n, 20.3n | 71.0n, 15.4n | 98.0n, 22.3n | 89.2n, 17.5n |
| wiki block sort | 38.5n, 19.6n | 38.9n, 19.6n | 42.8n, 21.8n | 43.0n, 21.8n |
| bufferless block sort | 70.4n, 24.2n | 69.4n, 22.0n | 87.9n, 27.1n | 86.9n, 25.0n |

The rotation matters to the rotation partition and not to the encoded one. The
stable quick sort at 2^18 distinct, in milliseconds:

| rotation | by encoded | by rotation |
|---|---|---|
| reversal | 18.5 | 48.6 |
| trinity | 19.6 | 36.9 |
| gries-mills | 20.2 | 61.3 |
| helix | 19.7 | 55.3 |
| juggling | 20.5 | 61.4 |
| block cycle | 19.4 | 17.2 |
| optimized piston, 3 windows | 19.9 | 16.5 |
| auxiliary, n / 2 words | 15.4 | 14.8 |

The level heap sort over the encoded partition against the rotation partition,
both by reversal, in milliseconds, with the moves and comparisons an element at
2^18 distinct:

| buffer | 2^18 rotation | 2^18 encoded | 2^20 rotation | 2^20 encoded | moves 2^18, rotation then encoded |
|---|---|---|---|---|---|
| external half | 41.9 | 28.1 | 191 | 142 | 63.3n, 43.1n |
| external sqrt | 82.5 | 28.8 | 363 | 126 | 98.1n, 65.1n |
| in array sqrt | 78.4 | 28.4 | 353 | 128 | 102n, 71.5n |
| in array ten log | 87.9 | 28.4 | 414 | 131 | 102n, 74.2n |
| in array log | 95.6 | 29.6 | 447 | 129 | 96.1n, 66.8n |

With repeats the 2^18 times are 42.1 to 27.6, 75.8 to 25.3, 75.5 to 25.8, 74.2 to
22.1 and 77.8 to 20.5, and the 2^20 times 199 to 137, 366 to 121, 339 to 119, 352
to 100 and 370 to 97. The buffer curve of the level heap record is flat: the log
of `n` buffer sorts in 28 ms at 2^18 and 129 at 2^20, level with the square root
buffers, against the index heap, one word per element, at 64 ms and 600 to 740
in this session, and the bufferless block sort at 42 and 200. The index heap is
slower than the earlier record's 41 and 281 on unchanged code, and random access
over 8 MB is where a busy machine hurts, so take it as an upper bound on how slow
it is; the other rows reproduce.

The block merge sort's partition extraction, at 2^20, in milliseconds with moves
an element, distinct then repeats:

| shape | scan | partition by rotation | by sliding | by encoded |
|---|---|---|---|---|
| wiki, reversal | 85.6 (42.8n), 76.9 (43.0n) | 91.3 (44.9n), 83.5 (48.0n) | 87.6 (43.5n), 81.9 (48.0n) | 86.9 (43.6n), 79.5 (46.5n) |
| wiki, auxiliary | 85.1 (43.6n), 76.6 (43.8n) | 86.1 (40.7n), 77.9 (44.4n) | 85.4 (44.4n), 78.0 (49.6n) | 86.2 (43.7n), 78.3 (47.3n) |
| grail sqrt, reversal | 83.3 (40.6n), 74.6 (40.6n) | 96.1 (48.1n), 82.6 (46.2n) | 86.1 (43.8n), 78.3 (46.0n) | 85.9 (47.8n), 77.7 (44.8n) |

The encoded rows were measured again after the trim, and are the same to within
noise: the extraction's passes have both kinds of element.

## The two cuts

The first measurement left two things open, and both are done. The encoded
partition now trims, and the stable quick sort makes its second pass only when
the first left nothing below the pivot.

The stable quick sort over the encoded and the rotation partition by reversal,
before and after both cuts, moves and comparisons an element and milliseconds:

| sort | 2^18 distinct | 2^18 repeats | 2^20 distinct | 2^20 repeats |
|---|---|---|---|---|
| encoded, before | 62.0n, 27.8n, 20.3 | 56.9n, 19.7n, 15.1 | 74.1n, 31.4n, 95.1 | 64.2n, 22.5n, 70.2 |
| encoded, after | 45.4n, 21.3n, 17.5 | 41.0n, 16.2n, 13.3 | 53.5n, 23.4n, 81.2 | 46.3n, 18.4n, 62.1 |
| rotation, before | 84.6n, 27.1n, 53.7 | 84.9n, 19.0n, 43.4 | 102n, 30.5n, 247 | 105n, 21.6n, 203 |
| rotation, after | 80.9n, 20.3n, 46.9 | 71.0n, 15.4n, 38.4 | 98.0n, 22.3n, 215 | 89.2n, 17.5n, 181 |

The second pass is the whole of it for the sort: a quarter of the moves, a fifth
to a quarter of the comparisons and 12 to 15 percent of the time. The skip alone,
measured in a copy of the sort before the trim was written, gave 45.6n moves at
2^18 distinct, and the sort now gives 45.4n, since every pass of a sort has both
kinds of element to move. The rotation partition gains less in moves, since it
paid little for the second pass already.
The sort over the encoded partition is now level with the wiki block sort at
2^18 and 7 percent faster at 2^20, and moves a little more than it, 45 to 53
elements against 38 to 43.

The partition alone, at 2^20, moves per element before the trim and after:

| shape | before | after |
|---|---|---|
| sorted, front then back | 1.94n | 0 |
| all in back | 2.00n | 0 |
| all in front | 0 | 0 |
| random half, either type in the minority | unchanged | unchanged |
| alternating elements | 3.03n | 3.52n |

Alternating elements went up and nothing was lost: in the old run the blocks came
out whole at a power of two, no loose front element was left, and the last
rotation had nothing to do. The trim takes a front element off the front and a
back one off the back, which shifts the blocks, leaves a block less one of loose
elements and has the rotation move half the array. The same shape starting with
a back element, which the trim leaves alone, measures 3.52n, so that is its cost
and the old figure was a lucky alignment of the sizes.

The level heap sort is where the trim pays, since its selection partitions
whole ranges where a boundary leaves little or nothing to move. The encoded
partition by reversal, before and after the trim, at 2^18 distinct, moves an
element and milliseconds:

| buffer | before | after |
|---|---|---|
| external half | 51.7n, 29.3 | 43.1n, 28.1 |
| external sqrt | 130n, 35.5 | 65.1n, 28.8 |
| in array sqrt | 127n, 34.9 | 71.5n, 28.4 |
| in array ten log | 137n, 36.1 | 74.2n, 28.4 |
| in array log | 135n, 35.8 | 66.8n, 29.6 |

At 2^20 the same shapes go from 160, 158, 167 and 166 ms to 126, 128, 131 and
129. The moves, which had risen above the rotation partition's on the small
buffers, are now below it, a third fewer.

## Verdict

- The encoded partition is linear in practice: 3.3 moves and 1.1 comparisons
  an element on random data, flat from 2^12 to 2^20, between 0 and 4.5 moves
  and 1.0 and 1.1 comparisons on every shape tried, in a scratch of
  `8 log n` words. It is the stable partition to use when there is no scratch
  worth speaking of: 2.8 times fewer moves and 2.4 times less time than the
  rotation partition by reversal at 2^20, and the sort's time over it does not
  depend on the rotation, 15 to 21 ms across eight of them at 2^18.
- It does not beat a rotation partition that buffers. By block cycle, optimized
  piston or the auxiliary rotation the rotation partition wins by four to
  seventeen percent in the sort, with more moves for the first two, since a
  buffered rotation's moves are cheap sequential copies. The point of the encoded partition is nearly the
  same speed with no buffer of any size and no choice of rotation.
- The stable quick sort over it is a stable sort of 17.5 ms at 2^18 elements and
  81 at 2^20, in place but for a logarithm of words, level with the wiki block
  sort at 2^18 and 7 percent faster at 2^20, 42 percent of the bufferless block
  sort's time, and four and a half to five times the standard library's. It is the cheap form
  of the level sort, which cost 86 and 391 ms: a stable quick sort with a median
  of three pivot where the level sort took exact medians. It moves a little more
  than a block sort, 45 to 53 elements against 38 to 43, and compares about as
  much, 21 to 23 against 20 to 22.
- The level heap sort is where the axis pays. The level build's selection
  partitions the whole range once a depth, and with a small buffer that is
  `log (n / buffer)` depths; the rotation partition by reversal made them the
  record's hundred moves an element, and the encoded partition makes the time 2.8
  to 3.5 times less at 2^18 and at 2^20 for every buffer but the half, which
  takes 33 and 26 percent less, and its moves a third fewer, 98 to 65 an element
  at the square root. The buffer no longer sets the time: every small buffer sorts
  in 28 to 30 ms at 2^18.
- The block merge sort gains nothing. The encoded extraction is level with the
  scan to a few percent and removes the rotation partition's six to twelve
  percent penalty, as the sliding one did, which is the extraction axis record's
  verdict again: the extraction is a fortieth of the sort.

## Open

- The cutoff of 16 and the ninther at 128 are the middle of flat curves: 8 to 48
  measured within a tenth of each other, and the ninther's threshold from 64 to
  512 within a percent.
- The sweep moves every majority block when the minority is a few blocks, up to
  one move an element, and the last rotation moves the back blocks past fewer
  than `b` loose elements, up to one more. A short last block of its own in place
  of the loose elements would save the rotation if the sweep could carry a block
  of another length; not tried.
- The external buffer of the level heap sort is free while a level is built and
  could serve as the partition's scratch now that the role asks for scratch;
  for a carved buffer the pass would have to swap.
- Where `n / 2` words of scratch are on offer the rotation partition over the
  auxiliary rotation is the faster partition, 3.7 ms against 5.1 at 2^20, and the
  stable quick sort over it 14.8 ms against 15.4 at 2^18 distinct. The encoded
  partition is for when they are not on offer.
- A pass with little to move, not nothing, still costs its blocks: a minority of
  a 256th in front is 4.5n moves, the dearest shape, since every back block
  sweeps and rotates past the few front ones. Moving the few the other way would
  make it cheap; not tried.
