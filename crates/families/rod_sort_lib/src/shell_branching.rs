use array_vis_bench_traits::Complexity;
use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};

/// A branching strategy for shell-shell sort.
///
/// At each recursion level the algorithm:
///   1. Asks `branch(virtual_len)` how many interleaved sub-arrays to recurse into.
///   2. Recursively sorts each sub-array.
///   3. Optionally does an extra insertion-sort pass at `intermediate(virtual_len)`
///      (skip when it returns 0).
///   4. Does the final insertion-sort merge pass at the current stride.
///
/// `should_cut(virtual_len)` signals to skip recursion and do a direct
/// insertion-sort instead (the base case for variants with an explicit cutoff).
pub trait BranchingStrategy {
    const NAME: &'static str;
    const BIG_O: &'static str;

    /// True → skip recursion, insertion-sort this virtual sub-array directly.
    fn should_cut(virtual_len: usize) -> bool;

    /// How many interleaved sub-arrays to split into at this level.
    fn branch(virtual_len: usize) -> usize;

    /// Factor for an optional extra insertion-sort pass after recursive sorting,
    /// before the final merge.  Return 0 to skip.
    fn intermediate(virtual_len: usize) -> usize {
        let _ = virtual_len;
        0
    }
}

// ---------------------------------------------------------------------------
// Classic: binary split (branch = 2 always)
// ---------------------------------------------------------------------------
pub struct Classic;
impl BranchingStrategy for Classic {
    const NAME: &'static str = "classic";
    const BIG_O: &'static str = "O(N^2)";

    fn should_cut(_: usize) -> bool { false }
    fn branch(_: usize) -> usize { 2 }
}

// ---------------------------------------------------------------------------
// 3-parity: ternary split (branch = 3 always)
// ---------------------------------------------------------------------------
pub struct Parity3;
impl BranchingStrategy for Parity3 {
    const NAME: &'static str = "3-parity";
    const BIG_O: &'static str = "O(N^2)";

    fn should_cut(virtual_len: usize) -> bool { virtual_len < 2 }
    fn branch(_: usize) -> usize { 3 }
}

// ---------------------------------------------------------------------------
// Log-parity: branch = floor(log2(len))
// ---------------------------------------------------------------------------
pub struct LogParity;
impl BranchingStrategy for LogParity {
    const NAME: &'static str = "log-parity";
    const BIG_O: &'static str = "O(N^2)";

    fn should_cut(virtual_len: usize) -> bool { virtual_len < 16 }
    fn branch(virtual_len: usize) -> usize { (virtual_len as f64).log2() as usize }
    fn intermediate(virtual_len: usize) -> usize {
        Self::branch(virtual_len).saturating_sub(1)
    }
}

// ---------------------------------------------------------------------------
// Root-parity: branch = floor(sqrt(len))
// ---------------------------------------------------------------------------
pub struct RootParity;
impl BranchingStrategy for RootParity {
    const NAME: &'static str = "root-parity";
    const BIG_O: &'static str = "O(N^2)";

    fn should_cut(virtual_len: usize) -> bool { virtual_len <= 4 }
    fn branch(virtual_len: usize) -> usize { (virtual_len as f64).sqrt() as usize }
    fn intermediate(virtual_len: usize) -> usize {
        Self::branch(virtual_len).saturating_sub(1)
    }
}

// ---------------------------------------------------------------------------
// Optimised: branch = 32, with an intermediate pass at 15
// ---------------------------------------------------------------------------
pub struct Optimised;
impl BranchingStrategy for Optimised {
    const NAME: &'static str = "optimised";
    const BIG_O: &'static str = "O(N^2)";

    fn should_cut(virtual_len: usize) -> bool { virtual_len < 64 }
    fn branch(_: usize) -> usize { 32 }
    fn intermediate(_: usize) -> usize { 15 }
}

// ---------------------------------------------------------------------------
// Fibonacci: branch = index of nearest Fibonacci number to len
// ---------------------------------------------------------------------------
pub struct Fibonacci;
impl BranchingStrategy for Fibonacci {
    const NAME: &'static str = "fibonacci";
    const BIG_O: &'static str = "O(N^2)";

    fn should_cut(virtual_len: usize) -> bool { virtual_len < 16 }

    fn branch(virtual_len: usize) -> usize {
        // Find the index of the Fibonacci number closest to virtual_len.
        let mut a = 1usize;
        let mut b = 1usize;
        let mut idx = 1usize;
        while b < virtual_len {
            let tmp = b;
            b += a;
            a = tmp;
            idx += 1;
        }
        idx
    }

    fn intermediate(virtual_len: usize) -> usize {
        Self::branch(virtual_len).saturating_sub(1)
    }
}

// ── Composable annotations ──────────────────────────────────────────
//
// Every branching strategy currently declares O(N²) worst-case under
// shell-shell sort. Best case is O(N) for already-sorted input.
// Stability false at the algorithm level is set by the outer
// ShellShellSort composition.

macro_rules! impl_branching_annotations {
    ($ty:ty) => {
        impl HasTimeBounds for $ty {
            const WORST: Complexity = Complexity::N_SQUARED;
            const BEST: Complexity = Complexity::N1;
            const AVERAGE: Complexity = Complexity::N_SQUARED;
        }
        impl HasSpace for $ty {
            const SPACE: Complexity = Complexity::CONST;
        }
        impl HasStability for $ty {
            const STABLE: bool = true;
        }
    };
}

impl_branching_annotations!(Classic);
impl_branching_annotations!(Parity3);
impl_branching_annotations!(LogParity);
impl_branching_annotations!(RootParity);
impl_branching_annotations!(Optimised);
impl_branching_annotations!(Fibonacci);
