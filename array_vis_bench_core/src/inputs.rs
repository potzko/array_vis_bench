//! Canonical inputs for every algorithm category.
//!
//! An input is a plain generator fn registered once into `INPUTS` with the
//! categories it serves; `list_inputs` and `primary_input` filter the slice by
//! those tags. Every generator returns an array and one index, the split of a
//! rotation, the midpoint of a merge or the target of a quick select, and `0`
//! for the kinds that take no index. To add a shape: write the generator and
//! register an `InputEntry` with `#[distributed_slice(INPUTS)]`.

use linkme::distributed_slice;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::bench_registry::{Category, InputEntry, RunConfig, INPUTS};

// Partition and scratchspace operations take the same single array as a sort, so
// they share its inputs and its primary.
const ARRAY_ONLY: &[Category] = &[Category::Sort, Category::Partition, Category::Scratchspace];

// ── Array inputs: sort, partition and scratchspace ───────────────────────────

/// Uniformly random values in `[0, size)`, deterministic per `seed`.
fn shuffled(c: &RunConfig) -> Vec<usize> {
    let mut rng = StdRng::seed_from_u64(c.seed);
    let cap = c.size.max(1);
    (0..c.size).map(|_| rng.gen_range(0..cap)).collect()
}
#[distributed_slice(INPUTS)]
static SORT_SHUFFLED: InputEntry = InputEntry {
    name: "shuffled",
    categories: ARRAY_ONLY,
    generate: |c| (shuffled(c), 0),
    primary: true,
};

/// `[0, 1, ..., size-1]`, best case for many sorts.
fn ascending(c: &RunConfig) -> Vec<usize> {
    (0..c.size).collect()
}
#[distributed_slice(INPUTS)]
static SORT_ASCENDING: InputEntry = InputEntry {
    name: "ascending",
    categories: ARRAY_ONLY,
    generate: |c| (ascending(c), 0),
    primary: false,
};

/// `[size-1, ..., 1, 0]`, worst case for many sorts.
fn descending(c: &RunConfig) -> Vec<usize> {
    (0..c.size).rev().collect()
}
#[distributed_slice(INPUTS)]
static SORT_DESCENDING: InputEntry = InputEntry {
    name: "descending",
    categories: ARRAY_ONLY,
    generate: |c| (descending(c), 0),
    primary: false,
};

/// Every element equal to a fixed value, stress-tests "= pivot" handling.
fn all_same(c: &RunConfig) -> Vec<usize> {
    vec![42; c.size]
}
#[distributed_slice(INPUTS)]
static SORT_ALL_SAME: InputEntry = InputEntry {
    name: "all same",
    categories: ARRAY_ONLY,
    generate: |c| (all_same(c), 0),
    primary: false,
};

/// Ascending with `~sqrt(size)` random swaps. The "almost sorted" case
/// that favours adaptive sorts (natural merge, Timsort, ...).
fn nearly_sorted(c: &RunConfig) -> Vec<usize> {
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
#[distributed_slice(INPUTS)]
static SORT_NEARLY_SORTED: InputEntry = InputEntry {
    name: "nearly sorted",
    categories: ARRAY_ONLY,
    generate: |c| (nearly_sorted(c), 0),
    primary: false,
};

/// Random values drawn from a tiny universe `[0, sqrt(size))`, heavy
/// duplicates, exposes equality handling.
fn few_unique(c: &RunConfig) -> Vec<usize> {
    let mut rng = StdRng::seed_from_u64(c.seed);
    let universe = ((c.size as f64).sqrt() as usize).max(1);
    (0..c.size).map(|_| rng.gen_range(0..universe)).collect()
}
#[distributed_slice(INPUTS)]
static SORT_FEW_UNIQUE: InputEntry = InputEntry {
    name: "few unique",
    categories: ARRAY_ONLY,
    generate: |c| (few_unique(c), 0),
    primary: false,
};

// ── Rotation inputs ──────────────────────────────────────────────────────────
//
// The index is the split. Each registered shape picks a different split
// position so the user can compare how algorithms behave on balanced vs.
// unbalanced rotations.

/// `[0, 1, ..., size-1]` split at `size / 2`. Balanced rotation.
fn mid_split(c: &RunConfig) -> (Vec<usize>, usize) {
    ((0..c.size).collect(), c.size / 2)
}
#[distributed_slice(INPUTS)]
static ROT_MID: InputEntry = InputEntry {
    name: "mid-split",
    categories: &[Category::Rotation],
    generate: mid_split,
    primary: true,
};

/// Split at `size / 4`. Smaller left block; many rotation algorithms
/// have asymmetric cost in this regime.
fn quarter_split(c: &RunConfig) -> (Vec<usize>, usize) {
    ((0..c.size).collect(), c.size / 4)
}
#[distributed_slice(INPUTS)]
static ROT_QUARTER: InputEntry = InputEntry {
    name: "quarter-split",
    categories: &[Category::Rotation],
    generate: quarter_split,
    primary: false,
};

/// Split at `3 * size / 4`. Mirror of quarter-split.
fn three_quarter_split(c: &RunConfig) -> (Vec<usize>, usize) {
    ((0..c.size).collect(), (3 * c.size) / 4)
}
#[distributed_slice(INPUTS)]
static ROT_THREEQ: InputEntry = InputEntry {
    name: "three-quarter-split",
    categories: &[Category::Rotation],
    generate: three_quarter_split,
    primary: false,
};

/// Split at index 1. The smallest non-trivial rotation; some algorithms
/// have catastrophic worst-case behaviour here.
fn near_start_split(c: &RunConfig) -> (Vec<usize>, usize) {
    ((0..c.size).collect(), 1.min(c.size))
}
#[distributed_slice(INPUTS)]
static ROT_NEAR_START: InputEntry = InputEntry {
    name: "near-start-split",
    categories: &[Category::Rotation],
    generate: near_start_split,
    primary: false,
};

// ── Merge inputs ─────────────────────────────────────────────────────────────
//
// The index is the midpoint. Each half is independently sorted; the merge has
// to interleave them.

/// Even values in left half, odd values in right half: every other
/// element comes from the opposite half, so the merge is maximally
/// interleaving.
fn interleaved_merge(c: &RunConfig) -> (Vec<usize>, usize) {
    let mid = c.size / 2;
    let mut v: Vec<usize> = (0..mid).map(|i| 2 * i).collect();
    v.extend((0..c.size - mid).map(|i| 2 * i + 1));
    (v, mid)
}
#[distributed_slice(INPUTS)]
static MERGE_INTERLEAVED: InputEntry = InputEntry {
    name: "interleaved",
    categories: &[Category::Merge],
    generate: interleaved_merge,
    primary: true,
};

/// Left half is `0..mid`, right half is `mid..size`. Already in order,
/// the merge has no real work to do beyond comparing.
fn concat_merge(c: &RunConfig) -> (Vec<usize>, usize) {
    let mid = c.size / 2;
    ((0..c.size).collect(), mid)
}
#[distributed_slice(INPUTS)]
static MERGE_CONCAT: InputEntry = InputEntry {
    name: "concat (already sorted)",
    categories: &[Category::Merge],
    generate: concat_merge,
    primary: false,
};

/// Left half holds the upper values, right half the lower: worst case
/// for a merge that scans left-to-right.
fn reversed_halves(c: &RunConfig) -> (Vec<usize>, usize) {
    let mid = c.size / 2;
    let mut v: Vec<usize> = (mid..c.size).collect();
    v.extend(0..mid);
    (v, mid)
}
#[distributed_slice(INPUTS)]
static MERGE_REVERSED: InputEntry = InputEntry {
    name: "reversed-halves",
    categories: &[Category::Merge],
    generate: reversed_halves,
    primary: false,
};

// ── Small-sort inputs ────────────────────────────────────────────────────────
//
// Small sorts cap their size at 32; we register the same shapes as for
// sorts but clamp the size in each generator.

fn clamp_size(c: &RunConfig) -> RunConfig {
    RunConfig { size: c.size.min(32), seed: c.seed }
}

#[distributed_slice(INPUTS)]
static SS_SHUFFLED: InputEntry = InputEntry {
    name: "shuffled",
    categories: &[Category::SmallSort],
    generate: |c| (shuffled(&clamp_size(c)), 0),
    primary: true,
};

#[distributed_slice(INPUTS)]
static SS_DESCENDING: InputEntry = InputEntry {
    name: "descending",
    categories: &[Category::SmallSort],
    generate: |c| (descending(&clamp_size(c)), 0),
    primary: false,
};

// ── Quick-select inputs ──────────────────────────────────────────────────────
//
// The index is the target. Each shape picks a different target position so the
// visualiser can compare how a quickselect's recursion path changes when
// the target lands in different regions of the array.

/// `target = size / 2`, balanced split into both partitions on each step.
fn target_mid(c: &RunConfig) -> (Vec<usize>, usize) {
    (shuffled(c), c.size / 2)
}
#[distributed_slice(INPUTS)]
static QSEL_MID: InputEntry = InputEntry {
    name: "target-mid",
    categories: &[Category::QuickSelect],
    generate: target_mid,
    primary: true,
};

/// `target = 0`, always recurse left.
fn target_first(c: &RunConfig) -> (Vec<usize>, usize) {
    (shuffled(c), 0)
}
#[distributed_slice(INPUTS)]
static QSEL_FIRST: InputEntry = InputEntry {
    name: "target-first",
    categories: &[Category::QuickSelect],
    generate: target_first,
    primary: false,
};

/// `target = size - 1`, always recurse right.
fn target_last(c: &RunConfig) -> (Vec<usize>, usize) {
    (shuffled(c), c.size.max(1) - 1)
}
#[distributed_slice(INPUTS)]
static QSEL_LAST: InputEntry = InputEntry {
    name: "target-last",
    categories: &[Category::QuickSelect],
    generate: target_last,
    primary: false,
};

/// `target = size / 4`, left-leaning split.
fn target_quarter(c: &RunConfig) -> (Vec<usize>, usize) {
    (shuffled(c), c.size / 4)
}
#[distributed_slice(INPUTS)]
static QSEL_QUARTER: InputEntry = InputEntry {
    name: "target-quarter",
    categories: &[Category::QuickSelect],
    generate: target_quarter,
    primary: false,
};

#[cfg(test)]
mod tests {
    use crate::bench_registry::{list_inputs, primary_input, Category};

    const SORT_LIKE: [&str; 6] =
        ["all same", "ascending", "descending", "few unique", "nearly sorted", "shuffled"];

    // linkme leaves the order of a slice unspecified, so the lists are compared sorted.
    fn listed(category: Category) -> Vec<&'static str> {
        let mut names = list_inputs(category);
        names.sort_unstable();
        names
    }

    #[test]
    fn every_category_lists_its_inputs_and_names_one_primary() {
        let expected: [(Category, &[&str], &str); 7] = [
            (Category::Sort, &SORT_LIKE, "shuffled"),
            (Category::Partition, &SORT_LIKE, "shuffled"),
            (Category::Scratchspace, &SORT_LIKE, "shuffled"),
            (Category::Rotation, &["mid-split", "near-start-split", "quarter-split", "three-quarter-split"], "mid-split"),
            (Category::Merge, &["concat (already sorted)", "interleaved", "reversed-halves"], "interleaved"),
            (Category::SmallSort, &["descending", "shuffled"], "shuffled"),
            (Category::QuickSelect, &["target-first", "target-last", "target-mid", "target-quarter"], "target-mid"),
        ];
        for (category, names, primary) in expected {
            assert_eq!(listed(category), names, "inputs of {category:?}");
            assert_eq!(primary_input(category), primary, "primary of {category:?}");
        }
    }
}
