# Implementation plan: the masked partition

Status: built 2026-11-23, measurement pending. The question is whether the
blocks of `EncodedPartition` can be made from bits instead of cells: whether a
stable partition that needs no buffer and no distinct elements can replace the
first phase of the encoded partition, which gathers the region into blocks
through a buffer of `8 log n` elements, with permutations driven by bit masks
held in local variables. It can be built, it is stable and linear, and it is
tested to the standard of the encoded partition. How it compares in time is not
measured here: the record's numbers wait for one combined branch, on which this
feature and the mask merge are measured together so the catalog compiles once.

## What was built

- `MaskedPartition`, in `extraction_lib`, label `masked`, `MOVES = N1`,
  `scratch_size` 0. It keeps the trim, the ordering of the blocks by ranks
  written into the elements, and the last rotation of the encoded partition,
  which now live in `blocks.rs` as `trim`, `order_blocks` and `finish` for both
  partitions to share. Only the first phase, the making of the whole blocks,
  differs.
- `MaskedTablePartition`, label `masked table`, the same with one change to how
  a window is applied, below. It is not in the catalog: it is there to be
  measured against `MaskedPartition`, and the choice of which goes into the
  catalog is open.
- The two are `Masked<K>` over a `Walk`, `Popcount` and `Tabulated`, with
  `MaskedPartition` and `MaskedTablePartition` as aliases, so the windows, the
  masks and the places are one implementation and the walk is the one thing that
  differs.
- Catalog. `MaskedPartition` joins the `StableQuickSort` query, under every
  rotation, 15 entries, and gets a level heap study over the buffers and
  rotations of `LevelHeapEncoded`, 28 entries, 43 in all: the catalog goes from
  2534 entries to 2577. The block merge extraction study leaves the partition
  extraction over it out, as it did for the encoded partition. The sliding
  partition stays out of the stable quick sort, as before.

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
  two bytes a position of locals. In the first trials of this branch the table
  walk was faster by a clear factor, which the combined measurement is to
  confirm or not.

## Local memory

The arrays a window holds in locals, in bytes, at the provisional 16 words, which
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

## Sizes, provisional

The mask length is capped at 16 words, 1024 bits, so the masks stay a constant
number of words, and `W = 1024 - 2b` fresh elements fit beside the two partials.
The block length is eight times the base two logarithm of the region, rounded
up, as for the encoded partition, and at most a quarter of the cap. Both are
provisional: they are the middle of curves that were flat in the first trials on
a machine under load, and the combined measurement is to draw the curve of `b`
and of the cap for both walks, with the bytes each costs, and settle them. A
larger cap bought a few percent for the table walk in those trials and cost bytes
in proportion.

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

## Measurement, pending

To be measured on the combined branch, with the method of the record of the
stable partition axis: best of five timed runs without the logger, logged moves
and comparisons per element, random `usize` keys below `8n` (distinct) and below
`n / 64` (repeats), and keys below 4 and below 16, at 2^18 and 2^20. The machine
that this was developed on has performance and efficiency cores, and a run that
the scheduler places on an efficiency core, or that shares a core with other
work, takes up to twice as long; pin the run to an idle performance core and
repeat it before trusting a difference.

- One partition, masked, masked table and encoded: random half, sorted, all
  front, all back, front minority 1/16 and 1/256, back minority 1/16 and 1/256.
- `StableQuickSort` over each, by the reversal rotation and the best scratch free
  rotation, on all four inputs, with the wiki block sort and the standard
  library stable sort as references.
- The level heap study rows.
- The curve of `b` and of the cap, for both walks, with the bytes of each.

One thing to check at the same time. Moving the trim and the rest of the encoded
partition's driver into shared functions made its gather loop markedly slower in
the first trials until `gather` was kept out of line, and a comment on it says by
how much. The encoded rows of the combined measurement should be compared with
the record of the stable partition axis, 5.05 ms for one partition of 2^20
random elements, to confirm that the shared driver costs it nothing.

## Open

- Which walk goes into the catalog. `MaskedPartition`, the walk by popcount, is
  there now, the one that holds less than the encoded buffer, with
  `MaskedTablePartition` selectable by type for the measurement. If the table
  wins by more than the bytes it costs, it can take the label `masked` and the
  catalog entries, or join them, with the count change kept small.
- The cap and the block length, above.
- The first phase is the cost of the walk, not of the moves: the first phase
  writes an element about once, and the question is how to walk a cycle without
  the load and popcount of each step waiting on the one before. Two cycles walked
  at once would hide it where a window has two of them, which a random window
  seldom does.
- The trim and the rotation of the last phase are the encoded partition's, with
  its open points: the sweep of the majority blocks and the rotation of the back
  blocks past the loose elements.
