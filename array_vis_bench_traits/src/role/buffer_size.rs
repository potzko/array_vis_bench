//! `BufferSize`, how many cells a buffer gets for an array of `n` elements: the
//! size axis of a block merge sort's buffer, carved from the array or held
//! outside it. A size reads the length alone, never the elements, so picking one
//! makes no comparison the logger would have to show. `GROWTH` is its order, from
//! which the bounds of a sort that uses it follow.

use crate::Complexity;

pub trait BufferSize {
    /// The order of growth of `cells(n)`.
    const GROWTH: Complexity;

    /// Cells for an array of `n` elements.
    fn cells(n: usize) -> usize;
}
