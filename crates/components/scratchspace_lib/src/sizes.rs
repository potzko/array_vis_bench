//! The `BufferSize` fillers, the sizes a block merge sort's buffer comes in.
//! `Sqrt` is what WikiSort carves and GrailSort's dynamic buffer allocates, room
//! for a block of any merge, and `TwoSqrt` is twice that. `Log` and
//! `Fixed<CELLS>` fall behind the blocks, 512 being the static buffer of both
//! sorts. `Half` is the size of a plain merge sort's buffer and `Sixteenth` a
//! linear one an eighth as big.

use array_vis_bench_traits::role::BufferSize;
use array_vis_bench_traits::Complexity;

/// `isqrt(n)` cells, as many as a block of the largest merge holds.
pub struct Sqrt;
impl BufferSize for Sqrt {
    const GROWTH: Complexity = Complexity::SQRT_N;

    fn cells(n: usize) -> usize { n.isqrt() }
}

/// `2 isqrt(n)` cells, a block and as much again.
pub struct TwoSqrt;
impl BufferSize for TwoSqrt {
    const GROWTH: Complexity = Complexity::SQRT_N;

    fn cells(n: usize) -> usize { 2 * n.isqrt() }
}

/// `log2(n)` cells, rounded down: the small merges fit it, the blocks of the
/// large ones outgrow it.
pub struct Log;
impl BufferSize for Log {
    const GROWTH: Complexity = Complexity::LOG_N;

    fn cells(n: usize) -> usize { n.checked_ilog2().unwrap_or(0) as usize }
}

/// `CELLS` cells whatever the length, so the blocks of a large merge outgrow it.
pub struct Fixed<const CELLS: usize>;
impl<const CELLS: usize> BufferSize for Fixed<CELLS> {
    const GROWTH: Complexity = Complexity::CONST;

    fn cells(_n: usize) -> usize { CELLS }
}

/// Half the array, rounded up.
pub struct Half;
impl BufferSize for Half {
    const GROWTH: Complexity = Complexity::N1;

    fn cells(n: usize) -> usize { n.div_ceil(2) }
}

/// A sixteenth of the array, rounded up.
pub struct Sixteenth;
impl BufferSize for Sixteenth {
    const GROWTH: Complexity = Complexity::N1;

    fn cells(n: usize) -> usize { n.div_ceil(16) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_size_reads_the_length() {
        assert_eq!([0, 99, 10_000].map(Sqrt::cells), [0, 9, 100]);
        assert_eq!([0, 99, 10_000].map(Fixed::<512>::cells), [512; 3]);
        assert_eq!([0, 99, 10_000].map(Half::cells), [0, 50, 5000]);
        assert_eq!([0, 99, 10_000].map(TwoSqrt::cells), [0, 18, 200]);
        assert_eq!([0, 1, 99, 10_000].map(Log::cells), [0, 0, 6, 13]);
        assert_eq!([0, 99, 10_000].map(Sixteenth::cells), [0, 7, 625]);
    }

    // A buffer holds a block of every merge exactly when its size keeps up with
    // the square root.
    #[test]
    fn only_the_constant_and_the_logarithm_fall_behind_the_square_root() {
        assert!([Sqrt::GROWTH, TwoSqrt::GROWTH, Half::GROWTH, Sixteenth::GROWTH].iter().all(|g| g.at_least(Complexity::SQRT_N)));
        assert!(!Fixed::<512>::GROWTH.at_least(Complexity::SQRT_N));
        assert!(!Log::GROWTH.at_least(Complexity::SQRT_N));
    }
}
