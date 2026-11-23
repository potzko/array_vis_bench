# Implementation plan: the mask merge

Status: built 2026-11-23 on the branch `mask_merge`, not folded, measurement
pending. The question is whether a stable merge with no scratch of its own can
beat symMerge's `O(n log n)` moves when the part of two runs that overlaps is
short, and how far a bit mask held in local variables takes it. `MaskRotationMerge`
merges a window of up to 1024 positions in one move an element, and cuts a longer
one by rotations until the pieces fit, so a merge of two runs of `n / 2` costs
`O(n log (n / cap))` moves. It is built, tested and in the catalog. It is not
measured on this branch: the user has this feature and the masked partition
measured together on one combined branch, so the catalog compiles once. The cap of
16 words is provisional until then. `bufferless_merge` and the block merge sort are
untouched.

## The question

Three places in the repo merge two runs without a buffer, all by symMerge over a
rotation: the `smaller-side` rotation merge, `bufferless_merge` under the `NoBuffer`
and `NoScratch` backings and the extraction's prefix sort, and so the bufferless
block merge sort. symMerge cuts a window at the middle of its shorter run, rotates,
and recurses down to single elements, so an element moves about `log n` times.
Where a window is short, a merge that only compares can record where each output
position gets its element, and a permutation applied by cycles then writes each
element once. The mask holds that record in a few words of the stack, so nothing is
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

The cap is provisional. A window of `w` positions costs a scan and a walk of the
cycles, and the walk is a chain of a mask load, a popcount and the element's load
that cannot overlap from one step to the next, so a pass over a window is dear per
move next to a rotation's swaps, which run sequentially. The mask pass costs about
the same per element at every window that fits the cache, so a larger cap removes
levels of rotating, each a pass of cheap swaps, and nothing else. It also hands the
mask more of the windows whose keys are mostly ties, where symMerge trims and
rotates long stretches of one key cheaply and the mask pays its pass for each of
them. The default is 16 words, 1024 positions, a mask of 384 bytes of stack and a
window of 8 KB of data that sits in the level one cache, and is the guess in the
middle of that tradeoff; the combined measurement picks it.

## Measurement pending

To run on the combined branch, with the earlier records' method: best of timed runs
without the logger, logged moves and comparisons per element, random `usize` keys
below `8n` (distinct), below `n / 64` (repeats) and below 16, at `2^18` and `2^20`,
pinned to a quiet core, the least of several whole runs per row since the machine is
noisy. Each of these is a scratch binary over the crates, not committed.

- The rotation merge sorts, top-down and bottom-up, by `naive`, `smaller-side` and
  `mask`, by the reversal rotation and a second one, with the standard library's
  stable sort and the wiki block sort as references. The naive merge is quadratic and
  wants smaller sizes or one run.
- One merge alone, moves and time an element, for windows of 64, 256 and 1024 and for
  long runs, under each merge.
- The cap curve, `WORDS` of 1 to 128, for one merge alone and for the sorts, on all
  three key shapes.
- The split against the fallback, `SPLIT` on and off at the default cap, for one
  merge past the cap and for the sorts.
- The block merge sort's bufferless shapes, `NoBuffer` with no keys and with in array
  keys, with `bufferless_merge` as symMerge and as the mask merge at a few caps, in a
  copy of `scratchspace_lib`, `extraction_lib` and `block_merge_sort_lib` whose
  `bufferless_merge` is switched by a build setting. The tests of the three copies
  pass over the mask merge. Nothing in the repo's `bufferless_merge` changes until the
  user decides from the number.

Moves are not a common currency between a rotation and the mask: a rotation's move
is a swap in a sequential run and a mask move is a dependent chain, so read the time
and not the moves when the two kinds are compared.

## Open

- Everything in the section above: the cap, the split against the fallback, the
  bufferless block sort number, the sort tables.
- Keys with few distinct values are where the mask is least likely to win, since a
  window of long runs is what symMerge does well. The mask can count the runs it
  holds, `popcount(m ^ (m >> 1))` over the words, and a window with few runs per
  position could go to the rotation merge. Not tried.
- A table of sources, a `u16` for each position filled in a pass over the mask, makes
  the cycles a chain of one load. In a scratch copy the leaf was faster for a window
  of 1024. It spends two bytes a position of local memory and is not driven by the
  mask alone, so it was not built.
- A galloping trim was tried in a scratch copy: it saves comparisons and no time that
  the noise did not hide, so the trim is a binary search.
- The cycles of two independent walks could be interleaved to overlap the two chains.
  Not tried.
- `bufferless_merge`, the block merge sort's `NoBuffer` shapes and `AuxiliaryMerge`
  over `NoBuffer` could take the mask merge. In the shape with keys the local merges
  are short and the block rolling is most of the cost, so the gain there is the
  smaller one.
- `WORDS` and `SPLIT` could be query axes if a curve in the catalog is wanted, at the
  price of 120 entries for each value.
- `verify/timing.tsv` is not touched. It is machine specific and `--bless` is the
  user's to run; the 135 entries are not in it.
