//! Heap layout — maps logical heap indices to physical array positions.
//!
//! Lets a single heap impl serve both forward orientation (heap occupies
//! the left of the array; sorted region grows from the right end) and
//! reversed (heap on the right; sorted region grows from the left).

pub trait Layout {
    /// Map logical heap index `i` to a physical array index, given the
    /// original array length `n`.
    fn phys(i: usize, n: usize) -> usize;
}

pub struct Forward;
impl Layout for Forward {
    #[inline(always)]
    fn phys(i: usize, _n: usize) -> usize {
        i
    }
}

pub struct Reverse;
impl Layout for Reverse {
    #[inline(always)]
    fn phys(i: usize, n: usize) -> usize {
        n - 1 - i
    }
}
