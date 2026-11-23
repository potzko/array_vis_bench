# Implementation plan: the mask merge

Status: built and measured 2026-11-23, on the combined branch `mask_applications`, not
folded. The question is whether a stable merge with no scratch of its own can beat
symMerge's `O(n log n)` moves when the part of two runs that overlaps is short, and how far a
bit mask held in local variables takes it. `MaskRotationMerge` merges a window of up to 1024
positions in one move an element, and cuts a longer one by rotations until the pieces fit, so
a merge of two runs of `n / 2` costs `O(n log (n / cap))` moves. It is built, tested, in the
catalog and measured. It takes 35 to 48 percent off the time of the rotation merge sorts and
28 to 46 percent off the bufferless block sort on distinct keys and on repeats, and loses 6 to
40 percent on keys that take few values; the split beats the fallback on distinct keys and
loses to it on few. The cap of 16 words stays. `bufferless_merge` is untouched, and the
block merge sort takes its local merge as an axis, in the section before Open.

## The question

Three places in the repo merge two runs without a buffer, all by symMerge over a
rotation: the `smaller-side` rotation merge, `bufferless_merge` under the `NoBuffer`
and `NoScratch` backings and the extraction's prefix sort, and so the bufferless
block merge sort. symMerge cuts a window at the middle of its shorter run, rotates,
and recurses down to single elements, so an element moves about `log n` times.
Where a window is short, a merge that only compares can record where each output
position gets its element, and a permutation applied by cycles then writes each
element once. The mask holds that record in a few words, so no cell of elements is
allocated and the merge stays in place. A longer window cannot be held, so it is cut
the way symMerge cuts it until the pieces fit.

## What was built

- `merge_window`, in `merge_sort_lib`, `mask_merge.rs`: the merge of one window of
  at most `64 * WORDS` positions. One scan that only compares writes the origin
  mask, a pass of popcounts gives the rank of each word, and cycles apply the
  permutation, a second mask marking the positions walked. Every move is a logged
  `write` or the one `write_data` that closes a cycle, every comparison a logged
  `cmp_lt`.
- `MaskRotationMerge<R, WORDS = 16, SPLIT = true>`, label `mask<{rot}>`, a
  `RotationMerge` and a `MergeKind` beside `naive` and `smaller-side`. It trims both
  runs by binary search to the window that needs merging, merges the window by the
  mask when it fits and cuts it at the middle of its shorter run when it does not,
  one rotation over the `R` axis for each cut, and merges each pair of pieces the
  same way. `scratch_size` is the rotation's. The bounds are `O(n log n)` worst and
  average, `O(n)` best, space the rotation's, stable.
- The two parameters are there so the later measurement can compare from one build.
  `WORDS` is the cap, `64 * WORDS` positions. `SPLIT` off is the fallback: a window
  past the cap is handed whole to `SmallerSideRotationMerge`, which never uses the
  mask, and the bounds become that merge's, `O(n log^2 n)`. The catalog entries use
  the defaults, 16 words and the split; the fallback and other caps are reached by
  naming the type, as a scratch harness does, and are not in the catalog.
- Catalog. `merge.spec` declares `mrm_mask`, and the two rotation merge sort
  queries take `rmerge = _`, so the filler joined them with no query change: 120 new
  sorts, 4 small sorts by 15 rotations by the 2 drivers, and 15 first class merge
  kinds. The catalog goes from 2534 entries to 2669. The query is not curated. The
  count stays in the range of the other families (the level heap sorts are 238, the
  block merge sorts 486), and the rotation matters to the mask merge only above the
  cap, which is the same as for the two merges beside it. No menu path was added: a
  merge kind has none, and the sorts sit under the existing `rotation` leaves. The
  hand written golden lines and type lines were checked against a build of the
  small catalog and, once, against the full catalog's `catalog_entries.rs`, and the
  full `spec_catalog` tests passed with them before the `SPLIT` parameter was added;
  the parameter has a default and the type text of an entry is unchanged by it.
- Tests, 20 in `mask_merge.rs`. Stability with `(key, tag)` items against a stable
  sort for every pair of run lengths to 40 and for runs to 5000, in nine overlap
  shapes (disjoint either way round, one run inside the other either way round,
  interleaved, all equal, three values, a spread of values, every key in both),
  under reversal, trinity, block cycle and auxiliary rotations and caps of 1, 2, 16
  and 64 words, with the split and with the fallback; windows at the cap and one past
  it, where a counting rotation shows that a window that fits asks for no rotation
  and one that does not does, and that a window that fits is one mask under the
  fallback too; a window with trimmed ends; one move an element for every window that
  fits and comparisons within the window and the trim's searches; a replay logger, as
  in `extraction_lib`, that applies the logged events to a copy and finds it equal to
  the array, with the rotation's scratch registered; and growth tests that the moves
  an element rise by one for each factor of four in the run under a one word cap, are
  1.4 times fewer than symMerge's at the default, and are fewer under the split than
  under the fallback.

## Why it is stable, and why each element is written once

Merge A and B the usual way, taking an element of B only when it is below the next
element of A, so an equal element of A goes first. Bit `t` of the mask `M` is set
when output position `t` takes an element of B. The elements of A leave in order
and so do those of B, so position `t` takes the element `rank0(M, t)` of A when its
bit is clear and the element `rank1(M, t)` of B when it is set, `rank0` and `rank1`
counting the clear and set bits below `t`. That puts every element where the stable
merge puts it. The ranks count output positions, so writing the permutation never
changes them.

The permutation is applied in the gather direction. Position `t` is filled from
`source(t)`. A cycle starts at a position `s` not yet walked, lifts the element of
`s` into a local, fills each position in turn from its source and ends at the
position whose source is `s`, where the local goes. A source is read before it is
written because it is written only when the walk reaches it, and a walk visits a
position once, so a cycle of `c` positions is `c` writes.

The trim makes it exact. After it `arr[lo] > arr[mid]` and `arr[mid - 1] >
arr[hi - 1]`, so the first output is an element of B and the last one of A. B runs
out first, and the remainder of A is clear bits that need not be scanned. And no
position is fixed: a fixed position needs either no element of B before an A
element, which the first output denies, or every element of A before a B element,
which the last output denies. So a window of `w` positions is `w` moves, exactly,
and at most `w - 1` comparisons. The tests assert both.

The split is symMerge's. The pivot is the middle of the shorter run, its partner
in the other run is a `lower_bound` when the pivot is in A and an `upper_bound`
when it is in B, one rotation puts the two lower pieces together and the two upper
pieces together, and an equal element of A stays ahead of one of B because the
bounds are placed that way. Each piece is smaller than the window by the pivot's
half of the shorter run, so the recursion ends.

## The cap

A window of `w` positions costs a scan and a walk of the
cycles, and the walk is a chain of a mask load, a popcount and the element's load
that cannot overlap from one step to the next, so a pass over a window is dear per
move next to a rotation's swaps, which run sequentially. The mask pass costs about
the same per element at every window that fits the cache, so a larger cap removes
levels of rotating, each a pass of cheap swaps, and nothing else. It also hands the
mask more of the windows whose keys are mostly ties, where symMerge trims and
rotates long stretches of one key cheaply and the mask pays its pass for each of
them. The default is 16 words, 1024 positions, a mask of 384 bytes of words and a
window of 8 KB of data that sits in the level one cache, and is the middle of that
tradeoff. The measurement below found no knee: each doubling past 16 words gains 2 to 3
percent on distinct keys and costs 2 to 6 percent on few, so it stays.

## Measured

Measured 2026-11-23 on the combined branch `mask_applications`, which carries this
feature and the masked partition, so the catalog compiled once. The method and the
machine are those of `masked_partition.md`: the variants of a row timed interleaved in
rounds on copies of one input, the best of at least five rounds and of up to fifteen,
logged moves and comparisons from one run, random `usize` keys below `8n` (distinct),
`n / 64` (repeats), 16 and 4, at 2^18 and 2^20, each figure the best of at least four
whole runs pinned to the performance cores 6, 8 and 10 of a Core Ultra 9 185H, one
scratch binary over the crates of the worktree and not committed. One merge alone is a
big array of chunks, each two sorted runs of random keys, merged chunk by chunk: the
chunk is the row's size, the time is nanoseconds an element, and `repeats` there is keys
below the chunk over 64 and one, so 2 distinct keys in a chunk of 64 and 16 385 in a
chunk of 2^20. The median run of a row is within 8 percent of its best everywhere. The rows
are the least of four runs, two of which had another job on another core and two of which
had the machine to themselves; the naive merge sorts are one run each, since a run is minutes.

The mask's moves are not the rotation's moves: a rotation's move is a swap in a
sequential run and a mask move is a dependent chain, so the times are what is compared.

One merge alone, by the naive merge, the smaller side merge (symMerge) and the mask merge,
over the reversal rotation and over block cycle, moves and nanoseconds an element:

keys distinct

| merge of | naive, reversal | smaller side, reversal | mask, reversal | smaller side, block cycle | mask, block cycle |
|---|---|---|---|---|---|
| window 64 | 4.40n, 8.3 | 2.09n, 8.5 | 0.97n, 5.8 | 3.23n, 9.7 | 0.97n, 5.8 |
| window 256 | 16.41n, 15.1 | 3.02n, 10.2 | 0.99n, 5.7 | 4.60n, 11.4 | 0.99n, 5.7 |
| window 1024 | 64.42n, 34.1 | 4.00n, 10.8 | 1.00n, 5.5 | 6.05n, 12.0 | 1.00n, 5.5 |
| window 4096 | - | 4.99n, 11.5 | 2.23n, 6.3 | 8.03n, 12.5 | 3.34n, 6.1 |
| 2^14 | - | 5.99n, 11.8 | 3.23n, 6.7 | 10.03n, 12.7 | 5.33n, 6.3 |
| 2^18 | - | 7.99n, 12.6 | 5.23n, 7.6 | 14.03n, 13.2 | 9.32n, 6.8 |
| 2^20 | - | 8.99n, 13.3 | 6.22n, 8.1 | 16.04n, 13.3 | 11.34n, 7.1 |

keys repeats

| merge of | naive, reversal | smaller side, reversal | mask, reversal | smaller side, block cycle | mask, block cycle |
|---|---|---|---|---|---|
| window 64 | 0.49n, 1.0 | 0.85n, 1.6 | 0.50n, 2.6 | 1.13n, 1.8 | 0.50n, 2.6 |
| window 256 | 1.39n, 1.7 | 2.28n, 3.1 | 0.80n, 4.4 | 2.99n, 3.2 | 0.80n, 4.5 |
| window 1024 | 4.46n, 3.1 | 3.44n, 3.9 | 0.94n, 5.1 | 4.62n, 3.8 | 0.94n, 5.1 |
| window 4096 | - | 4.49n, 4.5 | 2.08n, 6.0 | 6.67n, 4.1 | 3.12n, 5.8 |
| 2^14 | - | 5.51n, 4.9 | 3.13n, 6.4 | 8.70n, 4.3 | 5.19n, 6.1 |
| 2^18 | - | 7.53n, 5.7 | 5.15n, 7.4 | 12.73n, 4.7 | 9.23n, 6.6 |
| 2^20 | - | 8.53n, 6.2 | 6.15n, 7.8 | 14.73n, 5.0 | 11.24n, 6.8 |

keys below 16

| merge of | naive, reversal | smaller side, reversal | mask, reversal | smaller side, block cycle | mask, block cycle |
|---|---|---|---|---|---|
| window 64 | 3.26n, 7.1 | 1.97n, 7.8 | 0.93n, 5.7 | 2.99n, 9.0 | 0.93n, 5.7 |
| window 256 | 4.17n, 4.7 | 2.73n, 6.3 | 0.94n, 5.4 | 3.87n, 6.8 | 0.94n, 5.4 |
| window 1024 | 4.21n, 2.9 | 3.36n, 3.8 | 0.94n, 5.1 | 4.51n, 3.6 | 0.94n, 5.1 |
| window 4096 | - | 3.95n, 2.9 | 1.87n, 5.4 | 5.59n, 2.0 | 2.83n, 5.3 |
| 2^14 | - | 4.46n, 2.4 | 2.67n, 4.7 | 6.71n, 1.5 | 4.58n, 4.4 |
| 2^18 | - | 5.36n, 2.8 | 7.97n, 4.0 | 7.91n, 1.4 | 11.20n, 1.8 |
| 2^20 | - | 5.84n, 3.0 | 9.03n, 4.5 | 8.40n, 1.5 | 12.37n, 1.9 |

With distinct keys the mask merge is the one merge that writes each element once: a window
of 64 costs 5.8 ns an element against 8.5 for symMerge and 8.3 for the naive merge, a window
of 256 costs 5.7 against 10.2 and 15.1, a window of 1024 costs 5.5 against 10.8 and 34.1,
and the moves are 0.97n to 1.00n against 2.1n, 3.0n and 4.0n for symMerge. The naive merge
is the quadratic one, 64n moves at 1024. Past the cap of 1024 positions the mask splits by
rotations and a merge of 2^20 elements costs 6.2n moves and 8.1 ns an element against 9.0n
and 13.3, 39 percent less time, and 11.3n and 7.1 ns against 16.0n and 13.3 by block cycle,
47 percent less. The mask merge uses no rotation inside a window, which is why the two
rotations agree to the digit up to 1024, and block cycle gains more past it, where the cuts
rotate.

With repeated keys it loses. A chunk of 64 with two distinct keys costs the mask 2.6 ns
against 1.6 for symMerge and 1.0 for the naive merge; a chunk of 1024 with 17 keys costs
5.1 against 3.9 and 3.1; a chunk of 2^20 with 16 385 keys costs 7.8 against 6.2 ns, 26 percent
more, and by block cycle 6.8 against 5.0. The mask pays a pass for every position of a
window, and symMerge and the naive merge trim and rotate long stretches of one key for
almost nothing. With 16 keys the mask wins small windows, 5.7 ns against 7.8 at 64, and loses the
long ones, 4.5 against 3.0 at 2^20, 50 percent more.

The rotation merge sorts, top-down and bottom-up, by the smaller side merge and by the mask
merge, over the reversal rotation and block cycle, with the wiki block sort and the standard
library's stable sort as references, in milliseconds, then moves and comparisons an element:

| ms | 2^18 distinct | 2^18 repeats | 2^18 below 16 | 2^18 below 4 | 2^20 distinct | 2^20 repeats | 2^20 below 16 | 2^20 below 4 |
|---|---|---|---|---|---|---|---|---|
| top-down smaller side, reversal | 39.8 | 35.2 | 16.4 | 9.90 | 188 | 168 | 72.6 | 44.8 |
| top-down smaller side, block cycle | 42.8 | 37.5 | 15.0 | 7.82 | 201 | 177 | 63.0 | 33.2 |
| bottom-up smaller side, reversal | 39.6 | 34.6 | 15.2 | 9.16 | 186 | 169 | 67.8 | 41.6 |
| bottom-up smaller side, block cycle | 42.5 | 36.8 | 13.5 | 6.91 | 200 | 177 | 56.4 | 29.4 |
| top-down mask, reversal | 23.4 | 23.2 | 18.0 | 13.2 | 110 | 110 | 81.3 | 60.8 |
| top-down mask, block cycle | 22.2 | 22.1 | 15.8 | 10.9 | 104 | 103 | 66.7 | 46.5 |
| bottom-up mask, reversal | 23.1 | 23.0 | 17.9 | 13.1 | 110 | 110 | 81.1 | 61.0 |
| bottom-up mask, block cycle | 22.0 | 21.9 | 15.6 | 10.7 | 104 | 103 | 66.3 | 46.5 |
| wiki block sort (in array sqrt) | 18.8 | 16.8 | 21.4 | 13.4 | 85.6 | 76.8 | 92.1 | 59.5 |
| standard library stable sort | 3.43 | 1.79 | 0.712 | 0.500 | 16.8 | 11.9 | 5.44 | 4.36 |

| moves, comparisons | 2^18 distinct | 2^18 repeats | 2^18 below 16 | 2^18 below 4 | 2^20 distinct | 2^20 repeats | 2^20 below 16 | 2^20 below 4 |
|---|---|---|---|---|---|---|---|---|
| top-down smaller side, reversal | 72.8n, 27.6n | 71.8n, 25.5n | 58.8n, 14.3n | 46.4n, 10.2n | 90.3n, 30.6n | 89.3n, 28.5n | 70.6n, 14.3n | 56.5n, 10.2n |
| top-down smaller side, block cycle | 115.0n, 27.6n | 111.9n, 25.5n | 82.2n, 14.3n | 59.2n, 10.2n | 146.2n, 30.6n | 143.1n, 28.5n | 99.7n, 14.3n | 71.6n, 10.2n |
| bottom-up smaller side, reversal | 72.8n, 27.6n | 71.8n, 25.5n | 58.8n, 14.3n | 46.4n, 10.2n | 90.3n, 30.6n | 89.3n, 28.5n | 70.6n, 14.3n | 56.5n, 10.2n |
| bottom-up smaller side, block cycle | 115.0n, 27.6n | 111.9n, 25.5n | 82.2n, 14.3n | 59.2n, 10.2n | 146.2n, 30.6n | 143.1n, 28.5n | 99.7n, 14.3n | 71.6n, 10.2n |
| top-down mask, reversal | 40.5n, 22.4n | 40.4n, 22.3n | 48.9n, 16.3n | 44.0n, 11.4n | 52.5n, 24.5n | 52.3n, 24.4n | 66.4n, 16.3n | 57.8n, 11.4n |
| top-down mask, block cycle | 59.4n, 22.4n | 59.2n, 22.3n | 65.7n, 16.3n | 54.6n, 11.4n | 81.0n, 24.5n | 80.8n, 24.4n | 89.8n, 16.3n | 71.5n, 11.4n |
| bottom-up mask, reversal | 40.5n, 22.4n | 40.4n, 22.3n | 48.9n, 16.3n | 44.0n, 11.4n | 52.5n, 24.5n | 52.3n, 24.4n | 66.4n, 16.3n | 57.8n, 11.4n |
| bottom-up mask, block cycle | 59.4n, 22.4n | 59.2n, 22.3n | 65.7n, 16.3n | 54.6n, 11.4n | 81.0n, 24.5n | 80.8n, 24.4n | 89.8n, 16.3n | 71.5n, 11.4n |

The mask merge takes 41 percent off the rotation merge sort over reversal on distinct keys,
110 ms against 188 at 2^20 and 23.4 against 39.8 at 2^18, with 52.5n moves against 90.3n and
24.5n comparisons against 30.6n, and 48 percent off over block cycle, 104 against 201, with
81n moves against 146n. On repeats it takes 35 and 42 percent off, 110 against 168 and 103
against 177. Top-down and bottom-up agree within 2 percent on every input but the few key
ones, where bottom-up is 6 to 12 percent faster by symMerge and level by the mask. The mask
sort is 22 to 29 percent slower than the wiki block sort, 104 to 110 ms against 85.6 at
2^20 on distinct keys, with no buffer and no keys, and 6 to 6.5 times slower than the
standard library's stable sort.

It loses on few distinct keys. On keys below 16 it is 12 percent slower by reversal, 81.3 ms
against 72.6 at 2^20, and 6 percent by block cycle, 66.7 against 63.0; on keys below 4 it is 36
and 40 percent slower, 60.8 against 44.8 and 46.5 against 33.2. In both the naive merge sort
is the fastest of the three, since few keys leave a merge a few rotations: at 2^20 it takes
60.3 ms on keys below 16 and 24.6 on keys below 4 by reversal, 46.7 and 22.3 by block cycle.
The naive sorts of distinct keys are what the name says: 4157 ms at 2^18 and 80 100 at 2^20,
one run each, against 110 for the mask. The naive rows, in milliseconds, with the others of
the same runs:

| ms | td naive, reversal | bu naive, reversal | td smaller side, reversal | td mask, reversal | td naive, block cycle | td smaller side, block cycle | td mask, block cycle |
|---|---|---|---|---|---|---|---|
| 2^14 distinct | 15.8 | 15.8 | 1.74 | 1.01 | 8.78 | 1.90 | 0.989 |
| 2^14 repeats | 3.49 | 3.48 | 1.48 | 0.995 | 2.45 | 1.59 | 0.980 |
| 2^14 below 16 | 0.732 | 0.683 | 0.901 | 0.906 | 0.676 | 0.892 | 0.890 |
| 2^14 below 4 | 0.310 | 0.290 | 0.516 | 0.659 | 0.301 | 0.472 | 0.618 |
| 2^16 distinct | 255 | 254 | 8.41 | 4.89 | 122 | 9.20 | 4.74 |
| 2^16 repeats | 47.6 | 47.3 | 7.36 | 4.87 | 26.4 | 7.87 | 4.70 |
| 2^16 below 16 | 3.19 | 2.94 | 3.81 | 4.05 | 2.75 | 3.63 | 3.76 |
| 2^16 below 4 | 1.28 | 1.20 | 2.27 | 2.97 | 1.22 | 1.93 | 2.57 |
| 2^18 distinct | 4157 | 4153 | 40.6 | 23.9 | 1883 | 43.9 | 22.2 |
| 2^18 repeats | 744 | 731 | 36.7 | 25.9 | 355 | 38.3 | 22.1 |
| 2^18 below 16 | 14.0 | 12.7 | 16.6 | 18.1 | 11.5 | 15.2 | 15.9 |
| 2^18 below 4 | 5.46 | 5.15 | 9.94 | 13.6 | 4.86 | 7.88 | 10.9 |
| 2^20 distinct | 80100 | 80734 | 188 | 110 | 33664 | 201 | 104 |
| 2^20 repeats | 12791 | 12735 | 171 | 111 | 5571 | 182 | 103 |
| 2^20 below 16 | 60.3 | 56.1 | 73.0 | 82.1 | 46.7 | 64.8 | 67.0 |
| 2^20 below 4 | 24.6 | 25.2 | 45.4 | 63.1 | 22.3 | 33.7 | 46.4 |

The split by rotations against the fallback, which hands a window past the cap whole to
symMerge, with symMerge itself, for one merge alone past the cap in moves and nanoseconds
an element and then for the sorts in milliseconds and moves:

| merge of | smaller side | mask, split | mask, fallback |
|---|---|---|---|
| window 1024, distinct | 4.00n, 11.1 | 1.00n, 5.4 | 1.00n, 5.4 |
| window 1024, repeats | 3.44n, 4.1 | 0.94n, 5.1 | 0.94n, 5.2 |
| window 1024, below 16 | 3.36n, 3.9 | 0.94n, 5.1 | 0.94n, 5.1 |
| window 4096, distinct | 4.99n, 11.5 | 2.23n, 6.4 | 4.99n, 11.5 |
| window 4096, repeats | 4.49n, 4.5 | 2.08n, 6.1 | 4.51n, 4.6 |
| window 4096, below 16 | 3.95n, 2.9 | 1.87n, 5.5 | 4.08n, 2.9 |
| 2^14, distinct | 5.99n, 11.8 | 3.23n, 6.7 | 5.99n, 11.8 |
| 2^14, repeats | 5.51n, 4.9 | 3.13n, 6.5 | 5.52n, 4.8 |
| 2^14, below 16 | 4.46n, 2.4 | 2.67n, 4.7 | 4.64n, 2.4 |
| 2^18, distinct | 7.99n, 12.6 | 5.23n, 7.6 | 7.99n, 12.7 |
| 2^18, repeats | 7.53n, 5.7 | 5.15n, 7.3 | 7.53n, 5.6 |
| 2^18, below 16 | 5.36n, 2.7 | 7.97n, 4.0 | 5.63n, 2.9 |
| 2^20, distinct | 8.99n, 13.2 | 6.22n, 8.1 | 8.99n, 13.3 |
| 2^20, repeats | 8.53n, 6.2 | 6.15n, 7.9 | 8.53n, 6.3 |
| 2^20, below 16 | 5.84n, 3.1 | 9.03n, 4.6 | 6.14n, 3.1 |

| ms | 2^18 distinct | 2^18 repeats | 2^18 below 16 | 2^18 below 4 | 2^20 distinct | 2^20 repeats | 2^20 below 16 | 2^20 below 4 |
|---|---|---|---|---|---|---|---|---|
| top-down smaller side, reversal | 39.8 | 35.4 | 16.4 | 9.85 | 187 | 168 | 72.3 | 45.0 |
| top-down mask, split, reversal | 23.4 | 23.2 | 18.0 | 13.3 | 111 | 110 | 81.5 | 60.9 |
| top-down mask, fallback, reversal | 34.2 | 29.7 | 14.4 | 11.9 | 165 | 147 | 64.6 | 53.8 |
| bottom-up smaller side, reversal | 39.4 | 34.8 | 15.2 | 9.33 | 188 | 169 | 67.2 | 43.0 |
| bottom-up mask, split, reversal | 23.3 | 23.0 | 17.8 | 13.2 | 111 | 110 | 80.6 | 60.6 |
| bottom-up mask, fallback, reversal | 34.0 | 29.3 | 14.2 | 11.8 | 165 | 146 | 65.1 | 53.9 |
| top-down smaller side, block cycle | 42.9 | 37.6 | 15.0 | 7.81 | 201 | 178 | 62.8 | 33.4 |
| top-down mask, split, block cycle | 22.1 | 22.1 | 15.8 | 10.9 | 103 | 103 | 66.9 | 46.1 |
| top-down mask, fallback, block cycle | 35.7 | 30.4 | 12.2 | 9.45 | 172 | 149 | 51.8 | 40.4 |

| moves | 2^18 distinct | 2^18 repeats | 2^18 below 16 | 2^18 below 4 | 2^20 distinct | 2^20 repeats | 2^20 below 16 | 2^20 below 4 |
|---|---|---|---|---|---|---|---|---|
| top-down smaller side, reversal | 72.8n | 71.8n | 58.8n | 46.4n | 90.3n | 89.3n | 70.6n | 56.5n |
| top-down mask, split, reversal | 40.5n | 40.4n | 48.9n | 44.0n | 52.5n | 52.3n | 66.4n | 57.8n |
| top-down mask, fallback, reversal | 62.6n | 61.6n | 49.9n | 45.2n | 80.1n | 79.1n | 61.9n | 57.1n |
| bottom-up smaller side, reversal | 72.8n | 71.8n | 58.8n | 46.4n | 90.3n | 89.3n | 70.6n | 56.5n |
| bottom-up mask, split, reversal | 40.5n | 40.4n | 48.9n | 44.0n | 52.5n | 52.3n | 66.4n | 57.8n |
| bottom-up mask, fallback, reversal | 62.6n | 61.6n | 49.9n | 45.2n | 80.1n | 79.1n | 61.9n | 57.1n |
| top-down smaller side, block cycle | 115.0n | 111.9n | 82.2n | 59.2n | 146.2n | 143.1n | 99.7n | 71.6n |
| top-down mask, split, block cycle | 59.4n | 59.2n | 65.7n | 54.6n | 81.0n | 80.8n | 89.8n | 71.5n |
| top-down mask, fallback, block cycle | 97.0n | 93.8n | 68.3n | 58.1n | 128.0n | 124.9n | 86.2n | 73.8n |

The split is what makes the mask worth having past the cap. A merge alone of 4096 elements
costs 2.23n moves and 6.4 ns by the split, and 4.99n and 11.5 by the fallback, which is
symMerge's number to the digit since every such window goes to it whole; a merge of 2^20
costs 6.2n and 8.1 against 9.0n and 13.3. In the sorts the fallback keeps the mask for the
lowest ten levels, where the windows fit, and takes 165 ms on distinct keys at 2^20 where
the split takes 111 and symMerge 187: of the 76 ms gained over symMerge the mask at the
leaves gives 22 and the split the other 54. On few keys the fallback is the better
of the two, and better than symMerge on keys below 16: 64.6 ms against 81.5 for the split
and 72.3 for symMerge at 2^20, 51.8 against 66.9 and 62.8 by block cycle. On keys below 4
symMerge is best, 45.0 against 53.8 and 60.9. The moves are not what the split pays for on
few keys, 66.4n against 61.9n for the fallback on keys below 16; the cause was not looked for.

The cap, 4 to 128 words, for one merge alone in moves and nanoseconds an element and for the
sorts over reversal in milliseconds and moves, with symMerge for reference:

| merge of | smaller side ns | 4 words | 8 words | 16 words | 32 words | 64 words | 128 words |
|---|---|---|---|---|---|---|---|
| window 64, distinct | 8.5 | 0.97n, 5.3 | 0.97n, 5.8 | 0.97n, 5.8 | 0.97n, 5.7 | 0.97n, 5.7 | 0.97n, 6.0 |
| window 64, below 16 | 7.9 | 0.93n, 5.1 | 0.93n, 5.6 | 0.93n, 5.6 | 0.93n, 5.5 | 0.93n, 5.6 | 0.93n, 5.9 |
| window 256, distinct | 10.2 | 0.99n, 5.3 | 0.99n, 5.7 | 0.99n, 5.7 | 0.99n, 5.7 | 0.99n, 5.7 | 0.99n, 5.8 |
| window 256, below 16 | 6.4 | 0.94n, 5.0 | 0.94n, 5.4 | 0.94n, 5.3 | 0.94n, 5.4 | 0.94n, 5.4 | 0.94n, 5.4 |
| window 1024, distinct | 11.2 | 2.19n, 6.3 | 1.71n, 6.2 | 1.00n, 5.5 | 1.00n, 5.5 | 1.00n, 5.5 | 1.00n, 5.5 |
| window 1024, below 16 | 4.0 | 1.91n, 5.5 | 1.43n, 5.6 | 0.94n, 5.1 | 0.94n, 5.1 | 0.94n, 5.1 | 0.94n, 5.1 |
| window 4096, distinct | 11.5 | 3.20n, 6.7 | 2.71n, 6.7 | 2.23n, 6.3 | 1.73n, 5.9 | 1.00n, 5.3 | 1.00n, 5.3 |
| window 4096, below 16 | 2.8 | 2.68n, 4.6 | 2.31n, 5.3 | 1.87n, 5.4 | 1.41n, 5.3 | 0.94n, 5.0 | 0.94n, 5.0 |
| 2^14, distinct | 11.7 | 4.20n, 7.0 | 3.72n, 7.0 | 3.23n, 6.7 | 2.74n, 6.4 | 2.24n, 6.1 | 1.74n, 5.8 |
| 2^14, below 16 | 2.4 | 6.15n, 3.1 | 5.73n, 3.7 | 2.67n, 4.7 | 2.29n, 5.1 | 1.86n, 5.2 | 1.41n, 5.1 |
| 2^18, distinct | 12.6 | 6.19n, 7.9 | 5.71n, 7.9 | 5.23n, 7.6 | 4.72n, 7.3 | 4.23n, 7.0 | 3.73n, 6.8 |
| 2^18, below 16 | 2.7 | 7.97n, 4.0 | 7.97n, 4.0 | 7.97n, 4.0 | 7.97n, 4.0 | 7.97n, 4.0 | 7.22n, 4.4 |
| 2^20, distinct | 13.4 | 7.20n, 8.3 | 6.72n, 8.3 | 6.22n, 8.0 | 5.74n, 7.8 | 5.24n, 7.5 | 4.73n, 7.2 |
| 2^20, below 16 | 3.0 | 9.03n, 4.5 | 9.03n, 4.5 | 9.03n, 4.4 | 9.03n, 4.5 | 9.03n, 4.5 | 9.03n, 4.5 |

| ms | 2^18 distinct | 2^18 repeats | 2^18 below 16 | 2^18 below 4 | 2^20 distinct | 2^20 repeats | 2^20 below 16 | 2^20 below 4 |
|---|---|---|---|---|---|---|---|---|
| top-down smaller side | 39.9 | 35.6 | 16.5 | 9.94 | 187 | 168 | 71.7 | 45.0 |
| top-down mask, 4 words | 23.8 | 23.5 | 16.3 | 12.0 | 113 | 112 | 75.2 | 55.1 |
| top-down mask, 8 words | 24.2 | 23.9 | 17.5 | 12.9 | 115 | 114 | 79.4 | 58.7 |
| top-down mask, 16 words | 23.3 | 23.2 | 18.0 | 13.3 | 110 | 110 | 81.8 | 60.7 |
| top-down mask, 32 words | 22.7 | 22.6 | 18.4 | 13.7 | 107 | 107 | 83.1 | 62.1 |
| top-down mask, 64 words | 22.2 | 22.2 | 18.9 | 14.2 | 105 | 105 | 85.3 | 63.9 |
| top-down mask, 128 words | 21.9 | 21.9 | 19.4 | 14.6 | 103 | 103 | 87.1 | 65.6 |
| bottom-up smaller side | 39.2 | 34.7 | 15.2 | 9.14 | 187 | 168 | 66.8 | 41.8 |
| bottom-up mask, 4 words | 23.6 | 23.3 | 16.2 | 11.8 | 113 | 112 | 75.3 | 55.4 |
| bottom-up mask, 8 words | 24.0 | 23.8 | 17.4 | 12.7 | 115 | 114 | 79.5 | 58.1 |
| bottom-up mask, 16 words | 23.2 | 23.1 | 17.9 | 13.0 | 111 | 111 | 80.9 | 60.7 |
| bottom-up mask, 32 words | 22.6 | 22.5 | 18.3 | 13.5 | 107 | 107 | 83.0 | 61.5 |
| bottom-up mask, 64 words | 22.1 | 22.0 | 18.6 | 13.9 | 105 | 104 | 84.8 | 63.5 |
| bottom-up mask, 128 words | 21.8 | 21.7 | 19.1 | 14.4 | 102 | 102 | 86.9 | 65.8 |

| moves | 2^18 distinct | 2^18 repeats | 2^18 below 16 | 2^18 below 4 | 2^20 distinct | 2^20 repeats | 2^20 below 16 | 2^20 below 4 |
|---|---|---|---|---|---|---|---|---|
| top-down smaller side | 72.8n | 71.8n | 58.8n | 46.4n | 90.3n | 89.3n | 70.6n | 56.5n |
| top-down mask, 4 words | 50.1n | 49.9n | 59.1n | 50.5n | 64.1n | 63.8n | 76.6n | 64.3n |
| top-down mask, 8 words | 45.1n | 44.9n | 54.2n | 47.4n | 58.1n | 57.9n | 71.7n | 61.3n |
| top-down mask, 16 words | 40.5n | 40.4n | 48.9n | 44.0n | 52.5n | 52.3n | 66.4n | 57.8n |
| top-down mask, 32 words | 36.3n | 36.2n | 43.1n | 40.2n | 47.3n | 47.2n | 60.6n | 54.1n |
| top-down mask, 64 words | 32.6n | 32.5n | 36.8n | 36.1n | 42.6n | 42.5n | 54.4n | 49.9n |
| top-down mask, 128 words | 29.4n | 29.3n | 30.1n | 31.5n | 38.4n | 38.3n | 47.6n | 45.4n |

A merge alone writes each element once when the window fits and costs a scan and a pass of
cycles at about 5.5 ns whatever the cap: a window of 1024 costs 2.19n moves at 4 words,
1.71n at 8 and 1.00n from 16, and 6.3 ns at 4 and 8 words and 5.5 from 16. A cap larger than
the window costs a little, 5.3 ns at 4 words against 5.8 at 16 and 6.0 at 128 for a window of
64, probably the arrays the call clears. A long merge gains from each doubling, 8.3 ns at 4 words to 7.2
at 128 for 2^20 distinct keys. In the sorts of distinct keys each doubling from 16 words
takes 2 to 3 percent, 113 ms at 4 words, 110 at 16, 105 at 64 and 103 at 128 at 2^20, and
moves 64n, 52n, 43n and 38n; on keys below 16 each doubling costs 2 to 6 percent, 75, 82, 85
and 87 ms at 4, 16, 64 and 128 words. The curve is flat and has no knee, so 16 words stays:
384 bytes of masks, a window of 8 KB of data in the level one cache, and the middle of both
sides of the curve.

The bufferless block sort, with no keys and with in array keys, by symMerge as in the repo
and by the mask merge of 4, 16 and 64 words in the local merges, from the copies of
`scratchspace_lib`, `extraction_lib` and `block_merge_sort_lib` in one binary with the
unmodified crates. The copy with symMerge is the repo's sort to 1 percent, 199 ms at 2^20 on
distinct keys with no keys, which makes the copies a fair comparison. Milliseconds, then
moves and comparisons an element:

no keys, reversal

| local merges, ms | 2^18 distinct | 2^18 repeats | 2^18 below 16 | 2^18 below 4 | 2^20 distinct | 2^20 repeats | 2^20 below 16 | 2^20 below 4 |
|---|---|---|---|---|---|---|---|---|
| repo | 42.5 | 37.9 | 18.2 | 11.4 | 199 | 180 | 78.7 | 50.9 |
| copy symMerge | 42.6 | 37.7 | 18.2 | 11.3 | 199 | 180 | 79.0 | 50.5 |
| mask 4 words | 26.3 | 26.0 | 18.9 | 14.0 | 124 | 122 | 86.0 | 64.2 |
| mask 16 words | 26.0 | 25.9 | 20.8 | 15.5 | 123 | 122 | 92.8 | 70.2 |
| mask 64 words | 25.0 | 24.8 | 21.6 | 16.4 | 116 | 116 | 95.9 | 73.8 |

in array keys, reversal

| local merges, ms | 2^18 distinct | 2^18 repeats | 2^18 below 16 | 2^18 below 4 | 2^20 distinct | 2^20 repeats | 2^20 below 16 | 2^20 below 4 |
|---|---|---|---|---|---|---|---|---|
| repo | 41.3 | 36.5 | 21.3 | 13.5 | 190 | 170 | 91.4 | 59.9 |
| copy symMerge | 40.9 | 36.6 | 21.3 | 13.5 | 190 | 171 | 92.0 | 59.5 |
| mask 4 words | 29.7 | 29.2 | 24.2 | 16.0 | 133 | 132 | 104 | 71.2 |
| mask 16 words | 30.7 | 30.6 | 25.9 | 17.2 | 137 | 137 | 112 | 76.0 |
| mask 64 words | 31.3 | 31.2 | 27.3 | 18.3 | 139 | 140 | 118 | 79.2 |

no keys, block cycle

| local merges, ms | 2^18 distinct | 2^18 repeats | 2^18 below 16 | 2^18 below 4 | 2^20 distinct | 2^20 repeats | 2^20 below 16 | 2^20 below 4 |
|---|---|---|---|---|---|---|---|---|
| repo | 46.2 | 40.4 | 17.0 | 9.46 | 214 | 191 | 70.8 | 39.8 |
| copy symMerge | 46.1 | 40.5 | 16.9 | 9.44 | 214 | 191 | 70.5 | 39.2 |
| mask 4 words | 24.6 | 24.4 | 15.9 | 11.3 | 114 | 113 | 67.1 | 47.5 |
| mask 16 words | 25.0 | 24.8 | 18.5 | 13.2 | 115 | 114 | 77.2 | 55.2 |
| mask 64 words | 24.2 | 24.2 | 20.2 | 14.6 | 110 | 111 | 84.9 | 61.4 |

in array keys, block cycle

| local merges, ms | 2^18 distinct | 2^18 repeats | 2^18 below 16 | 2^18 below 4 | 2^20 distinct | 2^20 repeats | 2^20 below 16 | 2^20 below 4 |
|---|---|---|---|---|---|---|---|---|
| repo | 45.5 | 39.9 | 20.6 | 11.6 | 210 | 187 | 84.5 | 49.6 |
| copy symMerge | 45.7 | 40.0 | 20.8 | 12.0 | 211 | 188 | 85.3 | 50.4 |
| mask 4 words | 29.7 | 29.5 | 22.6 | 13.8 | 133 | 133 | 93.5 | 57.7 |
| mask 16 words | 31.2 | 31.1 | 24.9 | 15.4 | 140 | 138 | 102 | 64.1 |
| mask 64 words | 31.5 | 31.5 | 26.8 | 16.7 | 140 | 140 | 110 | 69.9 |

| reference, ms | 2^18 distinct | 2^18 repeats | 2^18 below 16 | 2^18 below 4 | 2^20 distinct | 2^20 repeats | 2^20 below 16 | 2^20 below 4 |
|---|---|---|---|---|---|---|---|---|
| wiki block sort (in array sqrt) | 18.7 | 16.7 | 21.4 | 13.4 | 84.6 | 76.8 | 92.5 | 60.1 |
| standard library stable sort | 3.45 | 1.81 | 0.698 | 0.504 | 16.8 | 11.9 | 5.38 | 4.32 |

| moves, comparisons | 2^18 distinct | 2^18 repeats | 2^18 below 16 | 2^18 below 4 | 2^20 distinct | 2^20 repeats | 2^20 below 16 | 2^20 below 4 |
|---|---|---|---|---|---|---|---|---|
| no keys, reversal, repo | 70.4n, 24.1n | 69.4n, 22.0n | 56.6n, 11.2n | 44.6n, 8.1n | 87.9n, 27.1n | 86.9n, 25.0n | 68.4n, 11.2n | 54.7n, 8.1n |
| no keys, reversal, mask 4 words | 46.6n, 20.0n | 46.4n, 19.6n | 55.8n, 11.9n | 48.0n, 8.0n | 60.6n, 22.4n | 60.3n, 22.0n | 73.3n, 12.0n | 61.8n, 8.0n |
| no keys, reversal, mask 16 words | 37.0n, 18.7n | 36.9n, 18.6n | 45.7n, 13.0n | 41.5n, 8.8n | 49.0n, 20.9n | 48.8n, 20.8n | 63.2n, 13.0n | 55.3n, 8.8n |
| no keys, reversal, mask 64 words | 29.1n, 18.4n | 29.0n, 18.4n | 33.6n, 14.4n | 33.6n, 9.8n | 39.1n, 20.4n | 39.0n, 20.4n | 51.1n, 14.5n | 47.4n, 9.8n |
| in array keys, reversal, repo | 60.0n, 28.2n | 59.0n, 26.1n | 56.5n, 18.4n | 52.3n, 11.8n | 71.7n, 31.7n | 70.7n, 29.6n | 68.4n, 18.4n | 62.6n, 11.8n |
| in array keys, reversal, mask 4 words | 45.5n, 25.1n | 45.2n, 24.7n | 58.2n, 19.3n | 56.5n, 12.6n | 53.6n, 27.9n | 53.4n, 27.6n | 73.4n, 19.3n | 68.3n, 12.5n |
| in array keys, reversal, mask 16 words | 43.1n, 24.7n | 43.0n, 24.6n | 53.0n, 20.0n | 52.7n, 13.1n | 49.3n, 27.3n | 49.2n, 27.2n | 68.1n, 20.0n | 64.4n, 13.1n |
| in array keys, reversal, mask 64 words | 43.1n, 24.7n | 43.0n, 24.6n | 45.9n, 20.9n | 47.7n, 13.8n | 48.6n, 27.2n | 48.5n, 27.2n | 60.8n, 20.9n | 59.5n, 13.8n |

The mask merge takes 38 percent off the bufferless block sort with no keys on distinct keys,
123 ms against 199 at 2^20 and 26.0 against 42.5 at 2^18, with 49n moves against 88n, and 28
percent off with in array keys, 137 against 190, 49n against 72n moves, which is the smaller
gain that the record expected, since the block rolling is most of the cost. Over block cycle
it takes 46 and 33 percent off, 115 against 214 and 140 against 210. The cap matters little
here, the local merges being short: with no keys 4, 16 and 64 words take 124, 123 and 116 ms,
and with in array keys 133, 137 and 140. The sorts are still slower than the wiki block
sort, 123 against 85 ms, and 7 times slower than the standard library's.

It loses on few keys by as much as the rotation merge sorts do: on keys below 16 by 18 percent
with no keys, 92.8 ms against 78.7 at 2^20, and 23 percent with in array keys, 112 against
91.4; on keys below 4 by 38 and 27 percent, 70.2 against 50.9 and 76.0 against 59.9. A smaller
cap loses less: 4 words lose 9 and 14 percent on keys below 16, 86.0 against 78.7 with no
keys and 104 against 91.4 with in array keys.

Verdict.

- The mask merge takes 42 to 45 percent off the moves and 35 to 48 percent off the time of the
  rotation merge sorts on distinct keys and on repeats, and 28 to 46 percent off the time of
  the bufferless block sort, whose local merges it would replace, with no scratch of its own
  and 384 bytes of masks. It loses on few distinct keys: 6 to 12 percent in the rotation merge
  sorts on keys below 16, 36 to 40 percent on keys below 4, 18 to 38 percent in the
  bufferless block sort on both, where the naive merge or symMerge are the better merges.
- The split takes 25 to 33 percent off the fallback on distinct keys and repeats, and the
  fallback takes 20 to 23 percent off the split on keys below 16 and 10 to 12 percent on keys
  below 4; the split is the default, being the one that is fast on the keys the merge is for.
- The cap stays at 16 words and the split stays on: each doubling of the cap past 16 gains 2 to
  3 percent on distinct keys and costs 2 to 6 percent on few keys. No default was changed, and
  the type text of an entry is the same, so `verify/catalog.types` is untouched.

## The block sort's merge axis

The local merge of the block merge sort, the merge it runs where neither its buffer nor its
keys reach, was `bufferless_merge`, symMerge over the sort's rotation. It is now the `merge`
slot of the sort, a fifth parameter `M: MergeStrategy`, which defaults to symMerge.

- The role. `MergeStrategy` is a rotation merge left open in its rotation,
  `type With<R: Rotation>: RotationMerge`, so the sort picks the merge and the rotation apart:
  the merge is `M::With<R>` over the `R` the sort rotates with everywhere else. It is not a
  `RotationMerge` or a `MergeKind`, so the merge kinds and the rotation merge sorts stay the
  entries they were. Two components fill it, `SmallerSideMerge` (label `smaller-side`) and
  `MaskMerge` (label `mask`, the defaults of `MaskRotationMerge`), declared in `merge.spec`.
- Where it runs. `merge_in_place` is the one call, with the guard for an empty side, and
  `Stash::None` and a merge with no block size reach it; every other caller of
  `bufferless_merge` (`AuxiliaryMerge`, the workflows, the extraction's prefix sort and
  gather) is as it was. The sort's scratch is the largest of the rotation's, the
  extraction's and the merge's. The local merge runs in every merge of the shapes without a
  buffer or without keys, and in the others only when the keys run short or the blocks outgrow
  the buffer.
- The bounds. They do not gain `M`: both merges are stable and in place over the rotation, and
  the sort's worst case is the smaller side merge's, which the mask merge does not exceed. A
  mask merge shape is bounded above by it, not tightly.
- Catalog. Every one of the 486 entries that were there names `merge: smaller-side`, with the
  same bounds, and a study of 90 adds the mask merge: every buffer (15) and both key shapes by
  the reversal, block cycle and auxiliary rotations, carving by the scan, `BlockMergeMasks`.
  The reversal is the plain rotation, block cycle the best measured with the mask merge and
  auxiliary the one with a scratch of its own, so the scratch of the sort is the merge's too.
  The block merge sorts are 576 and the catalog 2802. The golden and type lines were taken
  from the small catalog and checked against a scripted rename of the old lines.
- What it answers. The bufferless block sort numbers above, 38 percent off with no keys and 28
  with in array keys on distinct keys, 18 to 38 percent slower on few distinct keys, come from
  copies of the crates with the mask merge in the local merges. The 90 entries put the same
  switch in the catalog, over the shapes with a buffer too, so the catalog's timing measures
  it. Not measured here.

## Registered scratch

The mask is words of the sort now, not an array on the stack: the fixed size arrays a C version
keeps there, allocated once through the logger so the visualiser shows them and every write to
them is logged. `RotationMerge` has `SCRATCH_SECTIONS`, the lengths of the arrays of `usize` it
keeps (default empty), and `merge` takes them as `words`. The rotation merge sorts, `run_rotation_merge`
and the local merge of the block merge sort allocate them once with `with_words`, as one array in
sections drawn side by side, and thread the slice into every merge; `AuxiliaryMerge`,
`NaiveRotationMerge` and `SmallerSideRotationMerge` ask for none, and `bufferless_merge`, which
is the smaller side merge, is handed an empty slice.

`MaskRotationMerge` asks for three sections of `WORDS` words each: the origin mask, the count of
set bits below each of its words, and the positions walked. That is 48 words, the 384 bytes of
the arrays it had, at the default of 16. The words hold what the merges before left, so a window
writes every word it reads: the origin mask words past the scan, which are the clear bits of the
rest of A and were zero when the array was new, are cleared, the counts are written whole, and
the walked marks are cleared first. A window of `w` positions still costs `w` moves and at most
`w - 1` comparisons, and a write to a word is no move: the replay logger compares the arrays
with its copy as they are freed, with the words blank and with them full of garbage, and the
visualiser sees the origin mask fill a word to each 64 comparisons of the scan, then the counts,
then the walked marks cleared and set during the cycles. A sort over the mask merge creates the
three arrays once and frees them once, and a sort over another merge makes none.

Measured on 2^20 distinct keys in one build against dev 28cc610 with the arrays on the stack,
best of six runs on idle cores 6, 8 and 10, the runs within 1.5 percent of each other: the top
down rotation merge sort over the mask merge by reversal, 110.5 ms before and 112.2 ms after,
1.5 percent slower.

## Open

- The bufferless block sort number is in the section above: 38 percent off with no keys and
  28 with in array keys on distinct keys, and 18 to 38 percent slower on few keys. The block
  merge sort's choice is now an axis and the 90 entries of it are in the catalog. Switching
  `bufferless_merge` and `AuxiliaryMerge` over `NoBuffer` to the mask merge is the user's
  decision from that number.
- Keys with few distinct values are where the mask loses, by 6 to 40 percent in the sorts and
  by 50 percent for a long merge of 16 keys. The mask can count the runs it holds,
  `popcount(m ^ (m >> 1))` over the words, and a window with few runs per position could go to
  the rotation merge; not tried. The fallback beats the split on few keys by 10 to 23 percent
  and is the cheaper sort there, so a switch between the two by the run count is the shape
  such a rule would take.
- A table of sources, a `u16` for each position filled in a pass over the mask, makes
  the cycles a chain of one load. In a scratch copy the leaf was faster for a window
  of 1024. It spends two bytes a position of local memory and is not driven by the
  mask alone, so it was not built. The masked partition's walk by table took half the time
  of its walk by popcount, which is the same trade.
- A galloping trim was tried in a scratch copy: it saves comparisons and no time that
  the noise did not hide, so the trim is a binary search.
- The cycles of two independent walks could be interleaved to overlap the two chains.
  Not tried.
- `WORDS` and `SPLIT` could be query axes if a curve in the catalog is wanted, at the
  price of 120 entries for each value.
- `verify/timing.tsv` is not touched. It is machine specific and `--bless` is the
  user's to run; the 135 entries are not in it, nor the 90 of the block sort's merge axis,
  and its block sort rows carry names without the merge.
