//! Heap layout — maps logical heap indices to physical array positions.
//!
//! Lets a single heap impl serve both forward orientation (heap occupies
//! the left of the array; sorted region grows from the right end) and
//! reversed (heap on the right; sorted region grows from the left).

pub trait Layout {
    /// `true` for `Forward` (logical-0 root sits at physical-0), `false`
    /// for `Reverse` (logical-0 root sits at physical-`n-1`). Pairs with
    /// [`super::compare::Compare::ROOTWARD_IS_SMALLER_ORD`] so a flat
    /// `PartitionScheme`-driven heap build can pick its sort direction at
    /// compile time.
    const ROOTWARD_IS_LOW_PHYS: bool;

    /// Map logical heap index `i` to a physical array index, given the
    /// original array length `n`.
    fn phys(i: usize, n: usize) -> usize;
}

pub struct Forward;
impl Layout for Forward {
    const ROOTWARD_IS_LOW_PHYS: bool = true;

    #[inline(always)]
    fn phys(i: usize, _n: usize) -> usize {
        i
    }
}

pub struct Reverse;
impl Layout for Reverse {
    const ROOTWARD_IS_LOW_PHYS: bool = false;

    #[inline(always)]
    fn phys(i: usize, n: usize) -> usize {
        n - 1 - i
    }
}
