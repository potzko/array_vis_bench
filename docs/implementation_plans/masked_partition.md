# Implementation plan: the masked partition

Status: built and measured 2026-11-23, on the combined branch with the mask merge.
The question was whether the blocks of `EncodedPartition` can be made from bits instead of
cells: whether a stable partition that needs no buffer and no distinct elements can replace
the first phase of the encoded partition, which gathers the region into blocks through a
buffer of `8 log n` elements, with permutations driven by bit masks held in local
variables. It can be built, it is stable and linear, and it is tested to the standard of
the encoded partition. In time the walk by table matches the encoded partition, with no
buffer: 11 percent faster on one partition of random keys, level with it in the stable
quick sort on distinct keys and 2 to 9 percent faster on repeats and few keys, at 2210
bytes of locals; and it loses to it by 5 to 27 percent in the level heap sort and by a
factor of two to three on a split that leaves a small minority. The walk by popcount takes
1.8 times as long as the encoded partition in every row, at 290 bytes. The refactor that
moved the encoded partition's trim and finish into shared functions did not slow it: it is
5 percent faster. The walk to run is the walk by table, the constants stay, and the catalog
holds both walks, as an axis of the partition. The arrays of masks and counts are
registered words of the sort now, not locals, and cost nothing measurable: see
Registered scratch.

## What was built

- `MaskedPartition`, in `extraction_lib`, label `masked<popcount>`, `MOVES = N1`,
  `scratch_size` 0. It keeps the trim, the ordering of the blocks by ranks
  written into the elements, and the last rotation of the encoded partition,
  which now live in `blocks.rs` as `trim`, `order_blocks` and `finish` for both
  partitions to share. Only the first phase, the making of the whole blocks,
  differs.
- `MaskedTablePartition`, label `masked<table>`, the same with one change to how
  a window is applied, below. It is there to be measured against
  `MaskedPartition`, and is catalogued beside it.
- The two are `Masked<K>` over a `Walk`, `Popcount` and `Tabulated`, with
  `MaskedPartition` and `MaskedTablePartition` as aliases, so the windows, the
  masks and the places are one implementation and the walk is the one thing that
  differs.
- Catalog. The masked partition joins the `StableQuickSort` query, under every
  rotation, 15 entries, and gets a level heap study over the buffers and
  rotations of `LevelHeapEncoded`, 28 entries, 43 in all, by each of its walks.
  The block merge extraction study leaves the partition extraction over it out,
  as it did for the encoded partition. The sliding partition stays out of the
  stable quick sort, as before. The walks as an axis are below.

## The first phase, by windows

Call the elements that go to the front, below the pivot or not above it, front,
and the rest back, as in the record of the encoded partition. Each is one logged
comparison against the pivot. The block length is `b`. The region, after the trim
has left the ends in place, is kept as whole blocks of one type each, then a
front partial block of fewer than `b` front elements, then a back partial block
of fewer than `b` back elements, then the elements not yet read.

A window is the front partial, the back partial and the next `W` elements. Its
class mask is a small fixed array of 64 bit words in a local, bit `i` set when
element `i` of the window is a back element. The elements of the two partials are
known without a comparison, the new ones are compared once, and a word is built
in a register and stored once. With `f` the front elements of the window, `k` the
back elements, `F = (f / b) * b` and `K = (k / b) * b`, the window is permuted so
that it becomes

    [whole front blocks, F][whole back blocks, K][front partial][back partial]

by sending a front element of rank `r` among the front elements to `r` when `r`
is below `F` and to `F + K + (r - F)` otherwise, and a back element of rank `r`
among the back elements to `F + r` when `r` is below `K` and to
`F + K + (f - F) + (r - K)` otherwise. A rank is a popcount of the mask below the
position, with a small table of how many back elements lie below each word, so
the mask does not change while the elements move. The whole blocks grow the done
region, the two partials start the next window, and the last window is permuted
the same way with what is left. The partials are the unread end of the region
when the windows stop, which is where `finish` expects them: the ordering of the
blocks and one rotation of the back blocks past the front partial leave the
front elements in front and the back partial, already last, in place.

The permutation is applied by cycles, with one element carried in a local: take
the first position not done whose place is elsewhere, write what it holds to its
place, carry the element that was there and write it to its place in turn, until
the cycle closes. Each element that moves is written once, by `write_data`, the
same call the cycle sort uses, so every move is logged and a replay of the log
reproduces the array. The two walks differ in how they find the next place:

- `Popcount`: each place is worked out by popcount as the cycle is walked, and a
  second mask marks the positions done. The arrays are a few words.
- `Tabulated`: the places of the whole window are worked out first, into a table
  of 16 bit entries, a pass in which no entry waits for another, and the cycle
  walk reads the table, marking a position done by pointing its entry at itself.
  The walk of `Popcount` waits at every step on a load, a popcount and some
  selects before it can take the next; the table takes that out of the walk, at
  two bytes a position of locals. The combined measurement confirms it: the table
  walk takes half the time of the walk by popcount.

## Local memory

The arrays a window holds, in bytes, at 16 words, as they were when they were locals
(Registered scratch has the words they are now), which
is a window of up to 1024 elements, whatever the element type is:

| partition | arrays | bytes |
|---|---|---|
| `MaskedPartition` | classes 16 words, backs below each word 17 entries of 16 bits, positions done 16 words | 290 |
| `MaskedTablePartition` | classes 16 words, backs below each word, table of 1024 places of 16 bits | 2210 |
| `EncodedPartition` | its buffer, `8 ceil(log2 n)` elements, here `usize` at 2^20 elements | 1280 |

For `w` words the arrays are `18 w + 2` bytes by popcount and `138 w + 2` by
table. The encoded partition's buffer grows with the element type and with the
logarithm of the region, the masks do not: at 2^20 elements the walk by popcount
holds less than the encoded buffer for an element of 2 bytes or more, and the
walk by table for an element of 14 bytes or more. A test pins the two figures
of the masks and checks them against the encoded buffer of `usize` elements.

## Registered scratch

The arrays are words of the sort now, as the block partition's offset buffers are: the
fixed size arrays a C version keeps on the stack, allocated once through the logger so the
visualiser shows them and every write to them is logged.
`StablePartition` has `SCRATCH_SECTIONS`, the lengths of the arrays of `usize` it keeps
(default empty), and its methods take them as `words`. `Extraction` has the same and
`PartitionExtraction<P>` hands on `P`'s. `StableQuickSort`, `LevelHeapSort` and the
extraction of `BlockMergeSort` allocate them once with `with_words`, as one array in
sections drawn side by side, and thread the slice into every call.

`Masked<K>` asks for three sections: the mask of back elements, 16 words; the count of them
below each word, 17; and the walk's own array, `K::WORDS`, which replaces `LOCAL_BYTES`: the
positions done by popcount, 16 words, or the table of places, 1024, a word a position. That is
49 words and 392 bytes by popcount, 1057 words and 8456 bytes by table, where the arrays of
the table walk were 2210 bytes with places of 16 bits; the encoded buffer is 1280 bytes at
2^20 elements, so the walk by popcount still holds less and the walk by table more. The words hold what the
calls before left, so the mask and the counts are written whole for each window, the
walk by popcount clears the words of its window, and the table is filled before it is read.
The encoded and rotation partitions ask for none.

The tests: a sort over each walk creates its three arrays once and frees them once, however
many windows it partitions, and a sort over a partition without words makes none; the
replay logger compares the arrays with its copy as they are freed, which shows that no
write to them went unlogged, with the words blank and with them full of garbage a read
before a write would show; the mask fills a word to each 64 comparisons of the sweep
and the done marks follow the counts.

Measured on 2^20 elements in one build against dev 28cc610 with the arrays as locals, best of
six runs on idle cores 6, 8 and 10, the runs within 1.5 percent of each other:

| | before | after | after against before |
|---|---|---|---|
| one partition of a random half, table | 4.126 ms | 4.032 ms | 2.3 percent faster |
| one partition of a random half, popcount | 8.262 ms | 8.385 ms | 1.5 percent slower |
| stable quick sort, `masked<table>`, distinct keys | 81.4 ms | 78.3 ms | 3.8 percent faster |
| stable quick sort, `masked<popcount>`, distinct keys | 146.0 ms | 147.5 ms | 1.0 percent slower |

The table walk gets no slower for a table four times the size, 8 KB in the level one cache
beside a window of 8 KB of elements. The old arrays were zeroed whole for each window, 2 KB
for the table, and the words are written as far as the window goes, which may be why it
is a little faster; the cause was not isolated.

## Why it is stable, and why it is linear

Stable because within a window each class keeps its order: ranks follow
positions, and the front partial and the back partial sit at the start of the
window in scan order, so they are the earliest elements of their classes in it.
The whole blocks take each class's earliest elements, `F` of the fronts and `K` of
the backs, in order, and the partials take the latest, so across windows the done
region holds each class's elements in order, whole blocks, followed by the two
partials, which are the latest of their classes and start the next window as the
earliest of it. No step compares one element with another. After that the phases
of the encoded partition are unchanged and stable for the reasons its record
gives: the ordering of the blocks keeps each type's blocks in order, and the
loose elements are placed in order.

Linear because a window is `O(W + 2b)` work for `W` new elements, each element
read once and written at most once in its window, with the partials, fewer than
`2b` elements, written again in each window, and `b` is a logarithm of `n` times a
constant, so the first phase is `O(n)` moves and comparisons; the second and third
phases are those of the encoded partition. The code cells of the ordering need
`b - 1 >= ceil(log2(minority blocks))`, which a test checks for every region
size, and a window needs `W + 2b` bits to fit the masks, which the choice of `W`
as the cap minus `2b` gives.

## Sizes

The mask length is capped at 16 words, 1024 bits, so the masks stay a constant
number of words, and `W = 1024 - 2b` fresh elements fit beside the two partials.
The block length is eight times the base two logarithm of the region, rounded
up, as for the encoded partition, and at most a quarter of the cap. The
measurement draws both curves: the block length is flat from five to ten
logarithms and eight is kept; the cap is a trade of bytes for a few percent, 32 words
taking 7 to 9 percent off one partition and 2 to 8 percent off the stable quick sort
for twice the bytes, and 16 is kept, one number for both walks.

## Tests

The battery of the encoded partition, shared in `test_util.rs` so both call the
same code, for both walks: stability against a reference that spells out the
order, for `partition`, `partition_at_most` and `partition_three_way`, every
region size to 300 and some above, every shape of input (one type throughout,
either type in the minority, alternating blocks and elements, runs, sorted,
reversed, a lone block or element of the other type), four rotations, a scratch
the length asked for or longer. The windows are tested on their own, against the
statement of what a window becomes, and at every awkward size: one fresh element,
a few, one block, two, three and six, so windows end at every place a block can
end and in the middle of one. The walks are tested on random permutations of
every length to the cap, with a replay logger that checks that no move goes
unlogged. A partition of arrays of only two, three and four distinct values, which
is the point of the exercise, matches the reference around every pivot, and so
does the encoded partition on them. A pass with nothing to move costs its
comparisons alone, the moves per element stay flat from a few thousand elements
to 2^16 and below the rotation partition's, an empty scratch slice is enough, and
the sorts over the partition, `StableQuickSort` and `LevelHeapSort`, sort stably
and log every move.

## Measured

Measured 2026-11-23 on the combined branch `mask_applications`, which carries this
feature and the mask merge, so the catalog compiled once.

Method. The variants that are compared run interleaved: in each round one timed run
of each, on a fresh copy of one input, the order moved along by one place a round.
A row is the best round of at least five, and of up to fifteen (up to 25 for one
partition) while a variant's second best round was more than 5 percent above its
best. Moves and comparisons are from one logged run. Inputs are random `usize` keys
below `8n` (distinct), `n / 64` (repeats), 16 and 4, at 2^18 and 2^20, the pivot of a
partition the median value. Each figure is the best of at least two whole runs of the
harness, six for the partitions and ten for the regression check, one scratch binary over
the crates of the worktree, with the repo's release profile and flags, never committed. The partitions at `dev` `7d728f6`, the walks
generic over the words and the block length, and the copies of the bufferless block
sort that the mask merge's record uses are further crates of the same binary.

Machine. A Core Ultra 9 185H. Every run is pinned with `taskset` to one of the
performance cores 6, 8 and 10 (4.8 GHz at most), after checking that the core and
its sibling thread were under 15 percent busy. Cores 1 to 4 reach 5.1 GHz and ran
about 5 percent faster in the first merge runs, so those runs are left out and every
figure is from one kind of core. The partition, regression, rotation and curve rows
ran with nothing else running; the first runs of the sort rows ran two at a time on
different cores and were 1 percent slower than the same rows alone, which the sweep
and the repeat runs show.

Spread. The best times of one row agree from run to run within a few percent, and the
median run of a row is within 10 percent of its best in all but two of the 1754 rows of
the two records, a partition of 0.36 ms and a sort of 8.8 ms. A run in six or so is up to
20 percent slower on the rows of under half a millisecond, since a daemon wakes up for
longer than the row lasts; the best is the figure given. Differences under 3 percent are
not read.

One partition of 2^18 and 2^20 random keys, the pivot such that the front is the share
the row says, by the reversal rotation, in milliseconds. `popcount` is `MaskedPartition`
and `table` is `MaskedTablePartition`. Moves and comparisons an element are at 2^20 and
the same at 2^18 to within 0.01n; comparisons are the same for all three.

| input | moves encoded | moves masked | comparisons | 2^18 encoded | 2^18 popcount | 2^18 table | 2^20 encoded | 2^20 popcount | 2^20 table |
|---|---|---|---|---|---|---|---|---|---|
| random half | 3.28n | 2.65n | 1.10n | 1.11 | 2.01 | 0.988 | 4.61 | 8.25 | 4.11 |
| random half, n/64 keys | 3.27n | 2.64n | 1.10n | 1.12 | 2.01 | 0.992 | 4.60 | 8.23 | 4.11 |
| random half, 16 keys | 3.29n | 2.64n | 1.10n | 1.13 | 2.02 | 0.991 | 4.60 | 8.21 | 4.08 |
| random half, 4 keys | 3.28n | 2.64n | 1.10n | 1.12 | 2.01 | 0.989 | 4.59 | 8.21 | 4.07 |
| sorted, front then back | 0.00n | 0.00n | 1.00n | 0.105 | 0.105 | 0.119 | 0.423 | 0.426 | 0.483 |
| all front | 0.00n | 0.00n | 1.00n | 0.128 | 0.128 | 0.156 | 0.514 | 0.520 | 0.632 |
| all back | 0.00n | 0.00n | 1.00n | 0.082 | 0.082 | 0.082 | 0.335 | 0.333 | 0.335 |
| front minority 1/16 | 4.34n | 3.08n | 1.03n | 0.519 | 1.57 | 0.930 | 2.09 | 6.45 | 3.84 |
| front minority 1/256 | 4.47n | 3.10n | 1.02n | 0.383 | 1.44 | 0.886 | 1.57 | 5.99 | 3.67 |
| back minority 1/16 | 2.15n | 2.17n | 1.03n | 0.360 | 1.50 | 0.795 | 1.48 | 6.18 | 3.29 |
| back minority 1/256 | 1.96n | 2.07n | 1.02n | 0.236 | 1.35 | 0.737 | 1.07 | 5.66 | 3.18 |
| reversed, back then front | 2.54n | 1.63n | 1.06n | 0.365 | 1.00 | 0.642 | 1.54 | 3.54 | 2.70 |
| alternating, front first | 3.52n | 2.62n | 1.10n | 0.435 | 1.64 | 0.945 | 1.86 | 6.14 | 3.90 |
| alternating, back first | 3.52n | 2.62n | 1.10n | 0.438 | 1.63 | 0.946 | 1.85 | 6.08 | 3.87 |

The walk by table is 11 percent faster than the encoded partition on a random half, 4.11
ms against 4.61 at 2^20 and 0.99 against 1.11 at 2^18, with a fifth fewer moves, 2.65n
against 3.28n. The walk by popcount takes 1.8 times the encoded partition's time and twice
the table walk's, 8.25 ms. Where the split is lopsided the windows still cost the same and
the encoded partition does not: with a minority of a sixteenth the table walk is 1.8 to 2.2
times slower and with a 256th 2.3 to 3.0 times, since the encoded partition's blocks go by
for the price of reading them and the windows are permuted whatever they hold. A reversed
region, 1.63n moves against 2.54n, is still 1.8 times slower by table. Where nothing moves
the three are level at the read of the region, 0.42, 0.51 and 0.33 ms for sorted, all front
and all back, except that the table walk takes 14 and 23 percent longer on the first two,
which no move accounts for; the code around the trim was not looked at. The comparisons are
the encoded partition's, 1.10n on a random half.

The check that the refactor cost the encoded partition nothing. The encoded partition of
this branch against the encoded partition of `dev` `7d728f6`, interleaved in one binary
with ten runs, in milliseconds, and the stable quick sort over each by reversal:

| input | 2^18 branch | 2^18 dev | change | 2^20 branch | 2^20 dev | change |
|---|---|---|---|---|---|---|
| random half | 1.10 | 1.17 | -5.9% | 4.60 | 4.87 | -5.4% |
| random half, n/64 keys | 1.10 | 1.17 | -5.8% | 4.59 | 4.85 | -5.3% |
| random half, 16 keys | 1.10 | 1.16 | -5.8% | 4.57 | 4.83 | -5.4% |
| random half, 4 keys | 1.09 | 1.16 | -6.0% | 4.57 | 4.83 | -5.4% |
| sorted, front then back | 0.105 | 0.105 | +0.0% | 0.424 | 0.424 | +0.2% |
| all front | 0.128 | 0.128 | +0.0% | 0.515 | 0.516 | -0.1% |
| all back | 0.082 | 0.082 | +0.0% | 0.334 | 0.334 | -0.1% |
| front minority 1/16 | 0.511 | 0.504 | +1.4% | 2.10 | 2.09 | +0.4% |
| front minority 1/256 | 0.382 | 0.385 | -0.9% | 1.56 | 1.57 | -0.5% |
| back minority 1/16 | 0.350 | 0.358 | -2.0% | 1.48 | 1.50 | -1.4% |
| back minority 1/256 | 0.235 | 0.236 | -0.6% | 1.07 | 1.08 | -0.9% |
| reversed, back then front | 0.357 | 0.363 | -1.4% | 1.53 | 1.56 | -1.9% |
| alternating, front first | 0.430 | 0.436 | -1.4% | 1.84 | 1.86 | -1.1% |
| alternating, back first | 0.430 | 0.433 | -0.7% | 1.83 | 1.84 | -0.2% |

| stable quick sort, encoded, reversal | 2^18 branch | 2^18 dev | change | 2^20 branch | 2^20 dev | change |
|---|---|---|---|---|---|---|
| distinct | 17.3 | 18.0 | -4.1% | 78.7 | 82.3 | -4.3% |
| repeats | 12.9 | 13.2 | -2.6% | 59.7 | 62.0 | -3.7% |
| below 16 | 4.51 | 4.71 | -4.2% | 18.5 | 18.8 | -1.1% |
| below 4 | 2.41 | 2.57 | -6.0% | 9.91 | 10.4 | -4.9% |

It did not get slower. It is 5 percent faster on every random half, 4.60 against 4.87 ms at
2^20 and 1.10 against 1.17 at 2^18, and the same within 2 percent on every other shape; the
stable quick sort over it is 1 to 6 percent faster, 78.7 against 82.3 ms at 2^20 on
distinct keys. The shared trim, the shared finish and `gather` kept out of line are as fast
as the original, and a little faster. The stable partition axis record's 5.05 ms for one
partition of 2^20 random keys was measured with a browser open; this record's 4.60 is on the
same code, and the old figure is not comparable with the others here.

The stable quick sort over each partition, by the reversal rotation and by the best
rotation that needs no scratch, in milliseconds, with the wiki block sort and the standard
library's stable sort as references; `bridge` stands for the best rotation that needs no
scratch. The best scratch free rotation is `bridge`: by 1 to 3
percent over reversal for the table walk, level with reversal for the encoded partition.
Of the nine scratch free rotations all but juggling are within 4.5 percent of each other in
the sort and juggling is 8 percent behind, so little hangs on the choice; the sweep is below.

| ms | 2^18 distinct | 2^18 repeats | 2^18 below 16 | 2^18 below 4 | 2^20 distinct | 2^20 repeats | 2^20 below 16 | 2^20 below 4 |
|---|---|---|---|---|---|---|---|---|
| encoded, reversal | 17.3 | 12.9 | 4.54 | 2.44 | 79.2 | 60.3 | 18.5 | 9.98 |
| popcount, reversal | 32.0 | 23.7 | 8.13 | 4.18 | 144 | 112 | 33.9 | 17.1 |
| table, reversal | 17.9 | 12.0 | 4.19 | 2.21 | 80.8 | 56.3 | 18.1 | 9.08 |
| encoded, bridge | 17.1 | 12.6 | 4.40 | 2.37 | 77.8 | 58.4 | 18.0 | 9.64 |
| popcount, bridge | 31.7 | 23.2 | 7.91 | 4.10 | 143 | 110 | 32.6 | 16.7 |
| table, bridge | 17.8 | 11.7 | 4.06 | 2.15 | 79.1 | 54.7 | 17.3 | 8.78 |
| wiki block sort (in array sqrt) | 18.8 | 16.7 | 21.3 | 13.5 | 84.2 | 75.2 | 92.3 | 59.6 |
| bufferless block sort | 42.4 | 37.6 | 18.0 | 11.3 | 201 | 180 | 79.8 | 50.2 |
| standard library stable sort | 3.44 | 1.81 | 0.660 | 0.467 | 16.9 | 11.9 | 5.43 | 4.33 |

| moves, comparisons | 2^18 distinct | 2^18 repeats | 2^18 below 16 | 2^18 below 4 | 2^20 distinct | 2^20 repeats | 2^20 below 16 | 2^20 below 4 |
|---|---|---|---|---|---|---|---|---|
| encoded, reversal | 45.4n, 21.3n | 41.0n, 16.2n | 13.7n, 6.6n | 6.6n, 4.2n | 53.5n, 23.4n | 46.3n, 18.4n | 15.2n, 6.6n | 6.6n, 4.2n |
| popcount, reversal | 34.7n, 21.3n | 29.4n, 16.2n | 10.8n, 6.6n | 5.2n, 4.2n | 40.5n, 23.4n | 34.2n, 18.4n | 11.6n, 6.6n | 5.3n, 4.2n |
| table, reversal | 34.7n, 21.3n | 29.4n, 16.2n | 10.8n, 6.6n | 5.2n, 4.2n | 40.5n, 23.4n | 34.2n, 18.4n | 11.6n, 6.6n | 5.3n, 4.2n |
| encoded, bridge | 45.7n, 21.3n | 41.3n, 16.2n | 13.7n, 6.6n | 6.6n, 4.2n | 53.8n, 23.4n | 46.6n, 18.4n | 15.2n, 6.6n | 6.6n, 4.2n |
| table, bridge | 35.1n, 21.3n | 29.7n, 16.2n | 10.8n, 6.6n | 5.2n, 4.2n | 40.9n, 23.4n | 34.6n, 18.4n | 11.6n, 6.6n | 5.3n, 4.2n |
| wiki block sort (in array sqrt) | 38.5n, 19.6n | 38.8n, 19.5n | 56.5n, 18.4n | 52.3n, 11.8n | 42.8n, 21.8n | 43.0n, 21.7n | 68.4n, 18.4n | 62.6n, 11.8n |
| bufferless block sort | 70.4n, 24.1n | 69.4n, 22.0n | 56.6n, 11.2n | 44.6n, 8.1n | 87.9n, 27.1n | 86.9n, 25.0n | 68.4n, 11.2n | 54.7n, 8.1n |

The walk by table matches the encoded partition in the sort. By reversal it is 2 to 4
percent slower on distinct keys, 80.8 ms against 79.2 at 2^20 and 17.9 against 17.3 at
2^18, and 2 to 9 percent faster on repeats and on few keys, 56.3 against 60.3 ms on repeats
at 2^20 and 9.08 against 9.98 on keys below 4. It moves a quarter fewer elements, 40.5n
against 53.5n, and compares the same. The walk by popcount takes 1.7 to 1.9 times the
encoded partition's time in every row, 144 ms at 2^20 on distinct keys. The encoded and
table sorts are level with the wiki block sort on distinct keys, 78 to 81 against 84 ms,
1.3 times faster on repeats, and five to six times faster on keys below 16 and below 4,
where the wiki block sort moves 52 to 68 elements each. The standard library's stable sort
is 2 to 5 times faster than any of them, 4.7 times on distinct keys.

The rotation matters little to the sort over any of the three, since each partition does
one rotation of the blocks past the loose elements. The sort over all three partitions by
every rotation, in milliseconds. The rotations with a few hundred cells of scratch, block
cycle and optimized piston, take 7 percent off the table walk at 2^20 on distinct keys and
12 to 14 percent off the popcount walk and nothing off the encoded partition. The n over
two cells of the auxiliary rotation take 16 percent off the encoded partition, 14 off the
table walk and 49 off the popcount walk, which takes the buffered pass for every region
that fits those cells and is faster by it than by its own windows.

2^18 distinct

| rotation | encoded | popcount | table |
|---|---|---|---|
| reversal | 17.4 | 32.1 | 17.9 |
| gries-mills | 17.6 | 32.4 | 18.3 |
| helix | 17.7 | 32.0 | 18.0 |
| juggling | 18.3 | 32.7 | 18.6 |
| piston | 17.7 | 32.0 | 18.0 |
| bridge | 17.5 | 31.8 | 17.8 |
| drill | 17.7 | 32.1 | 18.1 |
| grail | 17.9 | 32.4 | 18.4 |
| contrev | 17.7 | 31.9 | 17.8 |
| trinity (8 cells) | 17.6 | 31.7 | 17.7 |
| block cycle | 17.6 | 27.9 | 16.6 |
| piston opt 3 windows | 17.3 | 27.2 | 16.6 |
| auxiliary (n/2) | 14.9 | 17.7 | 15.9 |

2^20 distinct

| rotation | encoded | popcount | table |
|---|---|---|---|
| reversal | 79.1 | 145 | 80.7 |
| gries-mills | 80.8 | 147 | 82.2 |
| helix | 81.0 | 145 | 80.9 |
| juggling | 84.2 | 148 | 85.3 |
| piston | 79.7 | 144 | 80.9 |
| bridge | 79.5 | 143 | 79.0 |
| drill | 81.3 | 145 | 81.7 |
| grail | 81.8 | 145 | 82.3 |
| contrev | 80.3 | 144 | 79.5 |
| trinity (8 cells) | 79.5 | 143 | 79.5 |
| block cycle | 80.0 | 128 | 75.3 |
| piston opt 3 windows | 78.3 | 125 | 74.7 |
| auxiliary (n/2) | 66.1 | 73.7 | 69.7 |

2^20 repeats

| rotation | encoded | popcount | table |
|---|---|---|---|
| reversal | 61.0 | 113 | 57.6 |
| gries-mills | 61.3 | 113 | 57.7 |
| helix | 60.4 | 112 | 56.4 |
| juggling | 64.6 | 117 | 61.4 |
| piston | 60.6 | 112 | 56.7 |
| bridge | 59.1 | 111 | 55.2 |
| drill | 60.6 | 113 | 57.1 |
| grail | 61.9 | 114 | 58.4 |
| contrev | 60.1 | 112 | 55.8 |
| trinity (8 cells) | 59.9 | 111 | 55.8 |
| block cycle | 59.2 | 107 | 55.1 |
| piston opt 3 windows | 58.5 | 105 | 55.0 |
| auxiliary (n/2) | 47.1 | 54.5 | 50.1 |

2^20 below 4

| rotation | encoded | popcount | table |
|---|---|---|---|
| reversal | 9.88 | 17.2 | 9.06 |
| gries-mills | 9.93 | 17.1 | 9.11 |
| helix | 10.4 | 17.2 | 9.06 |
| juggling | 11.2 | 17.9 | 9.91 |
| piston | 10.1 | 17.0 | 8.76 |
| bridge | 10.1 | 17.1 | 8.82 |
| drill | 10.4 | 17.1 | 9.09 |
| grail | 10.5 | 17.4 | 9.12 |
| contrev | 10.3 | 17.3 | 8.94 |
| trinity (8 cells) | 10.3 | 17.1 | 8.90 |
| block cycle | 10.2 | 17.2 | 8.75 |
| piston opt 3 windows | 10.0 | 16.9 | 8.70 |
| auxiliary (n/2) | 9.04 | 14.7 | 8.62 |

The level heap sort over each partition, with the buffers and rotations of the catalog's
study, in milliseconds, then moves and comparisons an element:

| ms | 2^18 distinct | 2^18 repeats | 2^18 below 16 | 2^18 below 4 | 2^20 distinct | 2^20 repeats | 2^20 below 16 | 2^20 below 4 |
|---|---|---|---|---|---|---|---|---|
| in array log, reversal, encoded | 27.5 | 20.1 | 14.7 | 10.1 | 124 | 93.4 | 61.8 | 43.2 |
| in array log, reversal, popcount | 49.3 | 38.2 | 18.8 | 11.1 | 230 | 177 | 75.8 | 47.0 |
| in array log, reversal, table | 34.1 | 23.8 | 16.0 | 10.6 | 158 | 110 | 64.7 | 45.2 |
| in array sqrt, reversal, encoded | 28.3 | 24.6 | 14.7 | 10.0 | 126 | 115 | 61.6 | 43.6 |
| in array sqrt, reversal, popcount | 44.7 | 42.2 | 18.8 | 11.1 | 199 | 199 | 76.3 | 46.6 |
| in array sqrt, reversal, table | 31.3 | 27.7 | 15.9 | 10.6 | 139 | 131 | 65.7 | 44.8 |
| external sqrt, reversal, encoded | 28.2 | 24.9 | 15.1 | 12.8 | 124 | 119 | 70.0 | 62.9 |
| external sqrt, reversal, popcount | 44.8 | 42.7 | 19.2 | 14.4 | 196 | 209 | 86.3 | 68.0 |
| external sqrt, reversal, table | 31.2 | 27.7 | 15.8 | 13.1 | 136 | 138 | 73.9 | 64.2 |
| external sqrt, auxiliary, encoded | 25.2 | 21.3 | 14.0 | 12.8 | 110 | 101 | 66.0 | 60.9 |
| external sqrt, auxiliary, popcount | 27.8 | 23.6 | 15.8 | 14.8 | 118 | 113 | 72.8 | 68.7 |
| external sqrt, auxiliary, table | 26.1 | 21.9 | 14.5 | 13.0 | 112 | 107 | 68.9 | 62.9 |

| moves, comparisons | 2^18 distinct | 2^20 distinct |
|---|---|---|
| in array log, reversal, encoded | 66.8n, 69.7n | 84.4n, 81.4n |
| in array log, reversal, popcount | 47.0n, 69.7n | 58.2n, 81.4n |
| in array sqrt, reversal, encoded | 71.5n, 56.5n | 82.1n, 64.1n |
| in array sqrt, reversal, popcount | 53.5n, 56.5n | 61.4n, 64.1n |
| external sqrt, reversal, encoded | 65.1n, 60.3n | 74.0n, 67.4n |
| external sqrt, reversal, popcount | 48.7n, 60.3n | 55.4n, 67.4n |

Here the encoded partition wins. The table walk is 27 percent slower on distinct keys,
158 against 124 ms at 2^20 with the log buffer, and 5 to 27 percent slower in every row by
reversal; the popcount walk takes 1.1 to 1.9 times as long. The masked partitions move a
third fewer elements, 58.2n against 84.4n, and lose in time all the same: the selections of the level build
partition ranges of every size, most of them small, and the walk per window is dearer than
the buffered pass for a region the encoded partition's buffer holds. That is a reading of
the shapes and not tested. With the auxiliary rotation, whose scratch of n over two cells
lets every partition that fits it take the buffered pass, the three are within 8 percent,
110, 118 and 112 ms.

The block length at the default cap of 16 words, one partition of random distinct keys,
the encoded partition's time above each table. The bytes of locals are 290 by popcount and
2210 by table at every row, the figures of the table above:

2^18, encoded 1.12 ms, 3.27n moves, 1.10n comparisons

| logs | b | window | moves | comparisons | popcount ms | table ms |
|---|---|---|---|---|---|---|
| 1 | 18 | 988 | 2.85n | 1.94n | 2.96 | 1.97 |
| 2 | 36 | 952 | 2.67n | 1.44n | 2.34 | 1.37 |
| 3 | 54 | 916 | 2.62n | 1.29n | 2.16 | 1.17 |
| 4 | 72 | 880 | 2.60n | 1.21n | 2.07 | 1.08 |
| 5 | 90 | 844 | 2.60n | 1.17n | 2.02 | 1.03 |
| 6 | 108 | 808 | 2.60n | 1.14n | 2.02 | 1.02 |
| 8 | 144 | 736 | 2.62n | 1.10n | 2.02 | 0.990 |
| 10 | 180 | 664 | 2.65n | 1.08n | 2.04 | 1.00 |
| 12 | 216 | 592 | 2.68n | 1.06n | 2.09 | 1.02 |
| 16 | 256 | 512 | 2.72n | 1.05n | 2.15 | 1.06 |
| 24 | 256 | 512 | 2.72n | 1.05n | 2.14 | 1.06 |

2^20, encoded 4.61 ms, 3.28n moves, 1.10n comparisons

| logs | b | window | moves | comparisons | popcount ms | table ms |
|---|---|---|---|---|---|---|
| 1 | 20 | 984 | 2.87n | 1.95n | 12.2 | 8.21 |
| 2 | 40 | 944 | 2.68n | 1.45n | 9.53 | 5.73 |
| 3 | 60 | 904 | 2.64n | 1.30n | 8.88 | 4.84 |
| 4 | 80 | 864 | 2.62n | 1.21n | 8.43 | 4.49 |
| 5 | 100 | 824 | 2.62n | 1.17n | 8.29 | 4.27 |
| 6 | 120 | 784 | 2.62n | 1.14n | 8.22 | 4.17 |
| 8 | 160 | 704 | 2.65n | 1.10n | 8.27 | 4.13 |
| 10 | 200 | 624 | 2.68n | 1.08n | 8.45 | 4.23 |
| 12 | 240 | 544 | 2.74n | 1.07n | 8.73 | 4.37 |
| 16 | 256 | 512 | 2.77n | 1.06n | 8.80 | 4.41 |
| 24 | 256 | 512 | 2.77n | 1.06n | 8.82 | 4.41 |

The curve is flat from 5 to 10 logarithms, within 4 percent of its minimum, and eight is in
the middle of it. A block of one logarithm costs the table walk twice the time, from its
1.95 comparisons an element, and past ten the window shrinks as `b` grows and the moves
rise. The block length stays at eight logarithms.

The window cap and the bytes of locals it takes, at blocks of four, eight and sixteen
logarithms. A row is the partition at that many words of the mask; `window` is the fresh
elements a window takes, and `b` is clamped to a quarter of the cap, which is why the
curves of four and eight words are short. The walks generic over the words and the block
length measure the same as the crate's at 16 words and eight logarithms, 8.24 and 4.11 ms
against 8.24 and 4.10 at 2^20:

2^18

| words | logs | b | window | moves | comparisons | popcount bytes | popcount ms | table bytes | table ms |
|---|---|---|---|---|---|---|---|---|---|
| 4 | 4 | 64 | 128 | 2.80n | 1.23n | 74 | 2.51 | 554 | 1.47 |
| 8 | 4 | 72 | 368 | 2.66n | 1.21n | 146 | 2.19 | 1106 | 1.19 |
| 8 | 8 | 128 | 256 | 2.75n | 1.11n | 146 | 2.27 | 1106 | 1.18 |
| 16 | 4 | 72 | 880 | 2.60n | 1.21n | 290 | 2.07 | 2210 | 1.09 |
| 16 | 8 | 144 | 736 | 2.62n | 1.10n | 290 | 2.01 | 2210 | 0.993 |
| 16 | 16 | 256 | 512 | 2.72n | 1.05n | 290 | 2.15 | 2210 | 1.06 |
| 32 | 4 | 72 | 1904 | 2.58n | 1.21n | 578 | 2.01 | 4418 | 1.05 |
| 32 | 8 | 144 | 1760 | 2.56n | 1.10n | 578 | 1.90 | 4418 | 0.910 |
| 32 | 16 | 288 | 1472 | 2.59n | 1.04n | 578 | 1.90 | 4418 | 0.889 |
| 64 | 4 | 72 | 3952 | 2.57n | 1.21n | 1154 | 1.98 | 8834 | 1.02 |
| 64 | 8 | 144 | 3808 | 2.53n | 1.10n | 1154 | 1.85 | 8834 | 0.872 |
| 64 | 16 | 288 | 3520 | 2.54n | 1.04n | 1154 | 1.81 | 8834 | 0.825 |
| 128 | 4 | 72 | 8048 | 2.55n | 1.21n | 2306 | 1.98 | 17666 | 1.05 |
| 128 | 8 | 144 | 7904 | 2.52n | 1.10n | 2306 | 1.83 | 17666 | 0.906 |
| 128 | 16 | 288 | 7616 | 2.50n | 1.04n | 2306 | 1.77 | 17666 | 0.834 |

2^20

| words | logs | b | window | moves | comparisons | popcount bytes | popcount ms | table bytes | table ms |
|---|---|---|---|---|---|---|---|---|---|
| 4 | 4 | 64 | 128 | 2.83n | 1.26n | 74 | 10.2 | 554 | 6.08 |
| 8 | 4 | 80 | 352 | 2.68n | 1.21n | 146 | 8.97 | 1106 | 4.90 |
| 8 | 8 | 128 | 256 | 2.79n | 1.12n | 146 | 9.16 | 1106 | 4.88 |
| 16 | 4 | 80 | 864 | 2.62n | 1.21n | 290 | 8.41 | 2210 | 4.49 |
| 16 | 8 | 160 | 704 | 2.65n | 1.10n | 290 | 8.24 | 2210 | 4.11 |
| 16 | 16 | 256 | 512 | 2.77n | 1.06n | 290 | 8.77 | 2210 | 4.38 |
| 32 | 4 | 80 | 1888 | 2.60n | 1.21n | 578 | 8.15 | 4418 | 4.31 |
| 32 | 8 | 160 | 1728 | 2.58n | 1.10n | 578 | 7.67 | 4418 | 3.76 |
| 32 | 16 | 320 | 1408 | 2.62n | 1.05n | 578 | 7.82 | 4418 | 3.73 |
| 64 | 4 | 80 | 3936 | 2.58n | 1.21n | 1154 | 8.05 | 8834 | 4.23 |
| 64 | 8 | 160 | 3776 | 2.55n | 1.10n | 1154 | 7.52 | 8834 | 3.63 |
| 64 | 16 | 320 | 3456 | 2.56n | 1.05n | 1154 | 7.34 | 8834 | 3.42 |
| 128 | 4 | 80 | 8032 | 2.58n | 1.21n | 2306 | 8.09 | 17666 | 4.40 |
| 128 | 8 | 160 | 7872 | 2.54n | 1.10n | 2306 | 7.46 | 17666 | 3.74 |
| 128 | 16 | 320 | 7552 | 2.53n | 1.05n | 2306 | 7.23 | 17666 | 3.46 |

A larger cap helps both walks and costs bytes in proportion. At eight logarithms the table
walk takes 4.11 ms at 16 words, 3.76 at 32 (9 percent less), 3.63 at 64 and 3.74 at 128 at
2^20; the popcount walk takes 8.24, 7.67 (7 percent less), 7.52 and 7.46. The bytes are
`18 w + 2` and `138 w + 2` for `w` words as the test pins, 290 and 2210 at 16, 578 and 4418
at 32, 1154 and 8834 at 64. A longer block helps a larger cap further, 3.42 ms at 64 words
and 16 logarithms, and the curve is flat to 24.

What the cap does to the sorts, which partition ranges of every size, the stable quick sort
and the level heap sort over the tuned partitions with eight logarithms, in milliseconds:

| ms | 2^18 distinct | 2^18 repeats | 2^18 below 16 | 2^18 below 4 | 2^20 distinct | 2^20 repeats | 2^20 below 16 | 2^20 below 4 |
|---|---|---|---|---|---|---|---|---|
| stable quick, encoded | 17.3 | 12.9 | 4.55 | 2.44 | 78.8 | 60.3 | 18.6 | 9.94 |
| stable quick, popcount, 4 words | 38.0 | 29.9 | 10.2 | 5.25 | 174 | 141 | 42.7 | 21.3 |
| stable quick, table, 4 words | 22.4 | 16.7 | 5.90 | 3.08 | 102 | 79.2 | 25.2 | 12.7 |
| stable quick, popcount, 8 words | 35.3 | 26.8 | 9.37 | 4.74 | 161 | 126 | 39.2 | 19.3 |
| stable quick, table, 8 words | 19.9 | 14.0 | 5.00 | 2.56 | 89.9 | 66.5 | 21.4 | 10.6 |
| stable quick, popcount, 16 words | 32.4 | 23.7 | 8.17 | 4.22 | 146 | 113 | 34.3 | 17.2 |
| stable quick, table, 16 words | 17.9 | 12.0 | 4.23 | 2.17 | 80.2 | 56.6 | 17.8 | 8.96 |
| stable quick, popcount, 32 words | 31.1 | 22.8 | 7.83 | 4.08 | 140 | 108 | 32.1 | 16.5 |
| stable quick, table, 32 words | 17.5 | 11.3 | 3.88 | 2.02 | 78.3 | 52.7 | 16.4 | 8.27 |
| stable quick, popcount, 64 words | 30.5 | 22.4 | 7.59 | 3.90 | 138 | 105 | 30.9 | 15.8 |
| stable quick, table, 64 words | 17.4 | 11.0 | 3.73 | 1.95 | 77.7 | 51.9 | 15.8 | 8.01 |
| stable quick, popcount, 128 words | 30.6 | 22.2 | 7.47 | 3.84 | 137 | 104 | 30.4 | 15.5 |
| stable quick, table, 128 words | 18.5 | 11.3 | 3.93 | 2.00 | 82.1 | 53.8 | 16.2 | 8.20 |
| level heap, in array log, encoded | 27.4 | 20.1 | 14.6 | 10.0 | 126 | 94.4 | 61.6 | 43.0 |
| level heap, in array log, table, 4 words | 39.6 | 30.5 | 17.5 | 11.4 | 185 | 141 | 71.1 | 47.6 |
| level heap, in array log, table, 8 words | 35.7 | 26.3 | 16.1 | 9.90 | 167 | 123 | 64.7 | 42.6 |
| level heap, in array log, table, 16 words | 33.6 | 23.3 | 15.1 | 9.49 | 157 | 108 | 61.2 | 40.7 |
| level heap, in array log, table, 32 words | 34.7 | 22.6 | 14.9 | 9.67 | 159 | 104 | 60.9 | 41.1 |
| level heap, in array log, table, 64 words | 36.4 | 22.4 | 14.7 | 9.43 | 165 | 103 | 59.1 | 40.2 |
| level heap, in array log, table, 128 words | 40.4 | 22.4 | 14.6 | 9.25 | 184 | 103 | 59.3 | 39.3 |

The stable quick sort wants 32 to 64 words and the level heap sort wants 16: at 32 words
the table walk takes 2 percent off the sort of distinct keys, 78.3 against 80.2 ms at 2^20,
and 5 to 8 percent off the sort of repeats and few keys, and the level heap sort is within
3.5 percent of what it was at 16; at 64 words the sort gains a little more and the level
heap sort loses 4 to 5 percent against 32; at 128 both lose. With 32 words the table walk is
level with the encoded partition on distinct keys, 78.3 against 78.8 ms, and 13 percent
faster on repeats, 52.7 against 60.3.

Verdict.

- The masked partition matches the encoded one and, by table, does it with no buffer
  and no choice of how many cells to hold: `scratch_size` is 0, so the sort over it
  and a rotation that needs no scratch allocates nothing, where the encoded partition
  takes 160 cells at 2^20, as an aux array through the logger. The cost is the locals,
  2210 bytes at 16 words against the encoded buffer's 1280 bytes of `usize` at 2^20,
  which grows with the element and the logarithm where the masks do not, and the
  shapes it loses on, lopsided splits and the small regions of the level heap sort.
- The walk to run is the walk by table. The walk by popcount holds 290 bytes and
  takes 1.8 times as long as the encoded partition in every row; it is the walk that
  holds less than the encoded buffer. Both are catalogued, below.
- The encoded partition did not get slower: it is 5 percent faster.
- The constants stay: eight logarithms is the middle of a flat curve, and 16 words is a
  choice of bytes between the 290 and 2210 of the 16 word default and the 578 and 4418 at
  32 words that bring the sort's few percent, which is the one number to settle with the walk.
  Measured, 32 words is better for the stable quick sort on all eight rows and level
  for the level heap sort, and is the number to take if the table walk is the one
  run and 4.4 KB of locals is not a cost.

## Both walks in the catalog

The walk is a slot of the partition: `Masked<{walk}>`, label `masked<{walk}>`, over the
role `MaskWalk`, which `Popcount` (label `popcount`) and `Tabulated` (label `table`) fill,
all declared in `extraction.spec` beside the partition. Wherever the masked partition was
catalogued both walks are: 15 more entries of the stable quick sort and 28 more of the level
heap sort, 43 in all, so the catalog goes from 2712 entries to 2755, and the block merge
extraction study leaves the partition extraction out for either walk, as `Masked<_>` in
the one `-` of its query. The entries are `masked<popcount>` and `masked<table>`, and the
partitions print the same two names in a failed test.

Why both. The measurement says which walk to run, the walk by table: it matches the
encoded partition, and the walk by popcount takes 1.8 times as long. But the two are the
two ends of one trade, and the popcount walk is the one true to the point of the
partition, which spends bits where the encoded partition spends cells: a mask of the
positions done in place of a table of 16 bit places, 290 bytes of locals against 2210. A
catalog that held the walk by table alone would show the partition at its speed and hide
what it costs to keep to bits, so it holds both, and the time of one against the bytes of
the other is read off the catalog as the buffers of the level heap sort are.

## Open

- The cap of 32 words, if the table walk is the one run and 4.4 KB of locals is not
  a cost: it is better for the stable quick sort on every row and level for the level heap
  sort. The bytes the tests pin, 290 and 2210, would change with it.
- The first phase is the cost of the walk, not of the moves: the first phase
  writes an element about once, and the question is how to walk a cycle without
  the load and popcount of each step waiting on the one before. Two cycles walked
  at once would hide it where a window has two of them, which a random window
  seldom does. The table walk answers it for the loads and still takes the cycle
  as a chain.
- A lopsided split costs the windows what a balanced one does, 2 to 3 times the encoded
  partition's time at a minority of a sixteenth to a 256th. A pass could count the
  minority first, or swap the few the other way, and keep the windows for the splits
  that are not lopsided; not tried.
- The level heap sort's small regions, where the table walk loses 5 to 27 percent: the
  encoded partition takes a buffered pass for a region its buffer holds and the masked
  partitions have no such path when the rotation brings no scratch. Reading it from the
  numbers with the auxiliary rotation, which gives them the buffered pass and closes the gap
  to 2 to 6 percent for the table walk; not tested directly.
- The trim and the rotation of the last phase are the encoded partition's, with
  its open points: the sweep of the majority blocks and the rotation of the back
  blocks past the loose elements.
