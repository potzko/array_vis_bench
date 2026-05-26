//! Canonical inputs for every algorithm category.
//!
//! Each "shape" is its own registered type implementing the matching
//! `*Input` trait from [`crate::bench_registry`]. The registries
//! `SORT_INPUTS`, `ROTATION_INPUTS`, `PARTITION_INPUTS`, `MERGE_INPUTS`,
//! and `SMALL_SORT_INPUTS` live there; this module is where the entries
//! are populated. To add a new shape: define a struct, implement the
//! trait, and register the entry with `#[distributed_slice]`.

use linkme::distributed_slice;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::bench_registry::{
    MergeInput, MergeInputEntry, QuickSelectInput, QuickSelectInputEntry, RotationInput,
    RotationInputEntry, RunConfig, SmallSortInput, SmallSortInputEntry, SortInput,
    SortInputEntry, MERGE_INPUTS, QUICK_SELECT_INPUTS, ROTATION_INPUTS, SMALL_SORT_INPUTS,
    SORT_INPUTS,
};

// ── Sort inputs ──────────────────────────────────────────────────────────────

/// Uniformly random values in `[0, size)`, deterministic per `seed`.
pub struct Shuffled;
impl SortInput for Shuffled {
    fn generate(c: &RunConfig) -> Vec<usize> {
        let mut rng = StdRng::seed_from_u64(c.seed);
        let cap = c.size.max(1);
        (0..c.size).map(|_| rng.gen_range(0..cap)).collect()
    }
}
#[distributed_slice(SORT_INPUTS)]
static SORT_SHUFFLED: SortInputEntry = SortInputEntry {
    name: "shuffled",
    generate: <Shuffled as SortInput>::generate,
    primary: true,
};

/// `[0, 1, …, size-1]` — best case for many sorts.
pub struct Ascending;
impl SortInput for Ascending {
    fn generate(c: &RunConfig) -> Vec<usize> {
        (0..c.size).collect()
    }
}
#[distributed_slice(SORT_INPUTS)]
static SORT_ASCENDING: SortInputEntry = SortInputEntry {
    name: "ascending",
    generate: <Ascending as SortInput>::generate,
    primary: false,
};

/// `[size-1, …, 1, 0]` — worst case for many sorts.
pub struct Descending;
impl SortInput for Descending {
    fn generate(c: &RunConfig) -> Vec<usize> {
        (0..c.size).rev().collect()
    }
}
#[distributed_slice(SORT_INPUTS)]
static SORT_DESCENDING: SortInputEntry = SortInputEntry {
    name: "descending",
    generate: <Descending as SortInput>::generate,
    primary: false,
};

/// Every element equal to a fixed value — stress-tests "= pivot" handling.
pub struct AllSame;
impl SortInput for AllSame {
    fn generate(c: &RunConfig) -> Vec<usize> {
        vec![42; c.size]
    }
}
#[distributed_slice(SORT_INPUTS)]
static SORT_ALL_SAME: SortInputEntry = SortInputEntry {
    name: "all same",
    generate: <AllSame as SortInput>::generate,
    primary: false,
};

/// Ascending with `~sqrt(size)` random swaps. The "almost sorted" case
/// that favours adaptive sorts (natural merge, Timsort, …).
pub struct NearlySorted;
impl SortInput for NearlySorted {
    fn generate(c: &RunConfig) -> Vec<usize> {
        let mut arr: Vec<usize> = (0..c.size).collect();
        if c.size < 2 {
            return arr;
        }
        let mut rng = StdRng::seed_from_u64(c.seed);
        let swaps = ((c.size as f64).sqrt() as usize).max(1);
        for _ in 0..swaps {
            let a = rng.gen_range(0..c.size);
            let b = rng.gen_range(0..c.size);
            arr.swap(a, b);
        }
        arr
    }
}
#[distributed_slice(SORT_INPUTS)]
static SORT_NEARLY_SORTED: SortInputEntry = SortInputEntry {
    name: "nearly sorted",
    generate: <NearlySorted as SortInput>::generate,
    primary: false,
};

/// Random values drawn from a tiny universe `[0, sqrt(size))` — heavy
/// duplicates, exposes equality handling.
pub struct FewUnique;
impl SortInput for FewUnique {
    fn generate(c: &RunConfig) -> Vec<usize> {
        let mut rng = StdRng::seed_from_u64(c.seed);
        let universe = ((c.size as f64).sqrt() as usize).max(1);
        (0..c.size).map(|_| rng.gen_range(0..universe)).collect()
    }
}
#[distributed_slice(SORT_INPUTS)]
static SORT_FEW_UNIQUE: SortInputEntry = SortInputEntry {
    name: "few unique",
    generate: <FewUnique as SortInput>::generate,
    primary: false,
};

// ── Rotation inputs ──────────────────────────────────────────────────────────
//
// The "input" for a rotation is (array, split_index). Each registered
// shape picks a different split position so the user can compare how
// algorithms behave on balanced vs. unbalanced rotations.

/// `[0, 1, …, size-1]` split at `size / 2`. Balanced rotation.
pub struct MidSplit;
impl RotationInput for MidSplit {
    fn generate(c: &RunConfig) -> (Vec<usize>, usize) {
        ((0..c.size).collect(), c.size / 2)
    }
}
#[distributed_slice(ROTATION_INPUTS)]
static ROT_MID: RotationInputEntry = RotationInputEntry {
    name: "mid-split",
    generate: <MidSplit as RotationInput>::generate,
    primary: true,
};

/// Split at `size / 4`. Smaller left block; many rotation algorithms
/// have asymmetric cost in this regime.
pub struct QuarterSplit;
impl RotationInput for QuarterSplit {
    fn generate(c: &RunConfig) -> (Vec<usize>, usize) {
        ((0..c.size).collect(), c.size / 4)
    }
}
#[distributed_slice(ROTATION_INPUTS)]
static ROT_QUARTER: RotationInputEntry = RotationInputEntry {
    name: "quarter-split",
    generate: <QuarterSplit as RotationInput>::generate,
    primary: false,
};

/// Split at `3 * size / 4`. Mirror of quarter-split.
pub struct ThreeQuarterSplit;
impl RotationInput for ThreeQuarterSplit {
    fn generate(c: &RunConfig) -> (Vec<usize>, usize) {
        ((0..c.size).collect(), (3 * c.size) / 4)
    }
}
#[distributed_slice(ROTATION_INPUTS)]
static ROT_THREEQ: RotationInputEntry = RotationInputEntry {
    name: "three-quarter-split",
    generate: <ThreeQuarterSplit as RotationInput>::generate,
    primary: false,
};

/// Split at index 1. The smallest non-trivial rotation; some algorithms
/// have catastrophic worst-case behaviour here.
pub struct NearStartSplit;
impl RotationInput for NearStartSplit {
    fn generate(c: &RunConfig) -> (Vec<usize>, usize) {
        ((0..c.size).collect(), 1.min(c.size))
    }
}
#[distributed_slice(ROTATION_INPUTS)]
static ROT_NEAR_START: RotationInputEntry = RotationInputEntry {
    name: "near-start-split",
    generate: <NearStartSplit as RotationInput>::generate,
    primary: false,
};

// ── Merge inputs ─────────────────────────────────────────────────────────────
//
// (array, mid). Each half is independently sorted; the merge has to
// interleave them.

/// Even values in left half, odd values in right half — every other
/// element comes from the opposite half, so the merge is maximally
/// interleaving.
pub struct InterleavedMerge;
impl MergeInput for InterleavedMerge {
    fn generate(c: &RunConfig) -> (Vec<usize>, usize) {
        let mid = c.size / 2;
        let mut v: Vec<usize> = (0..mid).map(|i| 2 * i).collect();
        v.extend((0..c.size - mid).map(|i| 2 * i + 1));
        (v, mid)
    }
}
#[distributed_slice(MERGE_INPUTS)]
static MERGE_INTERLEAVED: MergeInputEntry = MergeInputEntry {
    name: "interleaved",
    generate: <InterleavedMerge as MergeInput>::generate,
    primary: true,
};

/// Left half is `0..mid`, right half is `mid..size`. Already in order —
/// the merge has no real work to do beyond comparing.
pub struct ConcatMerge;
impl MergeInput for ConcatMerge {
    fn generate(c: &RunConfig) -> (Vec<usize>, usize) {
        let mid = c.size / 2;
        ((0..c.size).collect(), mid)
    }
}
#[distributed_slice(MERGE_INPUTS)]
static MERGE_CONCAT: MergeInputEntry = MergeInputEntry {
    name: "concat (already sorted)",
    generate: <ConcatMerge as MergeInput>::generate,
    primary: false,
};

/// Left half holds the upper values, right half the lower — worst case
/// for a merge that scans left-to-right.
pub struct ReversedHalves;
impl MergeInput for ReversedHalves {
    fn generate(c: &RunConfig) -> (Vec<usize>, usize) {
        let mid = c.size / 2;
        let mut v: Vec<usize> = (mid..c.size).collect();
        v.extend(0..mid);
        (v, mid)
    }
}
#[distributed_slice(MERGE_INPUTS)]
static MERGE_REVERSED: MergeInputEntry = MergeInputEntry {
    name: "reversed-halves",
    generate: <ReversedHalves as MergeInput>::generate,
    primary: false,
};

// ── Small-sort inputs ────────────────────────────────────────────────────────
//
// Small sorts cap their size at 32; we register the same shapes as for
// sorts but clamp the size in each generator.

fn clamp_size(c: &RunConfig) -> RunConfig {
    RunConfig { size: c.size.min(32), seed: c.seed }
}

pub struct SmallShuffled;
impl SmallSortInput for SmallShuffled {
    fn generate(c: &RunConfig) -> Vec<usize> {
        <Shuffled as SortInput>::generate(&clamp_size(c))
    }
}
#[distributed_slice(SMALL_SORT_INPUTS)]
static SS_SHUFFLED: SmallSortInputEntry = SmallSortInputEntry {
    name: "shuffled",
    generate: <SmallShuffled as SmallSortInput>::generate,
    primary: true,
};

// ── Quick-select inputs ──────────────────────────────────────────────────────
//
// (array, target). Each shape picks a different target position so the
// visualiser can compare how a quickselect's recursion path changes when
// the target lands in different regions of the array.

fn quick_select_shuffled(c: &RunConfig) -> Vec<usize> {
    <Shuffled as SortInput>::generate(c)
}

/// `target = size / 2` — balanced split into both partitions on each step.
pub struct QselMid;
impl QuickSelectInput for QselMid {
    fn generate(c: &RunConfig) -> (Vec<usize>, usize) {
        (quick_select_shuffled(c), c.size / 2)
    }
}
#[distributed_slice(QUICK_SELECT_INPUTS)]
static QSEL_MID: QuickSelectInputEntry = QuickSelectInputEntry {
    name: "target-mid",
    generate: <QselMid as QuickSelectInput>::generate,
    primary: true,
};

/// `target = 0` — always recurse left.
pub struct QselFirst;
impl QuickSelectInput for QselFirst {
    fn generate(c: &RunConfig) -> (Vec<usize>, usize) {
        (quick_select_shuffled(c), 0)
    }
}
#[distributed_slice(QUICK_SELECT_INPUTS)]
static QSEL_FIRST: QuickSelectInputEntry = QuickSelectInputEntry {
    name: "target-first",
    generate: <QselFirst as QuickSelectInput>::generate,
    primary: false,
};

/// `target = size - 1` — always recurse right.
pub struct QselLast;
impl QuickSelectInput for QselLast {
    fn generate(c: &RunConfig) -> (Vec<usize>, usize) {
        let n = c.size.max(1);
        (quick_select_shuffled(c), n - 1)
    }
}
#[distributed_slice(QUICK_SELECT_INPUTS)]
static QSEL_LAST: QuickSelectInputEntry = QuickSelectInputEntry {
    name: "target-last",
    generate: <QselLast as QuickSelectInput>::generate,
    primary: false,
};

/// `target = size / 4` — left-leaning split.
pub struct QselQuarter;
impl QuickSelectInput for QselQuarter {
    fn generate(c: &RunConfig) -> (Vec<usize>, usize) {
        (quick_select_shuffled(c), c.size / 4)
    }
}
#[distributed_slice(QUICK_SELECT_INPUTS)]
static QSEL_QUARTER: QuickSelectInputEntry = QuickSelectInputEntry {
    name: "target-quarter",
    generate: <QselQuarter as QuickSelectInput>::generate,
    primary: false,
};

pub struct SmallDescending;
impl SmallSortInput for SmallDescending {
    fn generate(c: &RunConfig) -> Vec<usize> {
        <Descending as SortInput>::generate(&clamp_size(c))
    }
}
#[distributed_slice(SMALL_SORT_INPUTS)]
static SS_DESCENDING: SmallSortInputEntry = SmallSortInputEntry {
    name: "descending",
    generate: <SmallDescending as SmallSortInput>::generate,
    primary: false,
};
