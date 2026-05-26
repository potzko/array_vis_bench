//! Heap arity — number of children per heap node.
//!
//! Used as a type-level parameter for `HeapSort<A>` so each arity produces a
//! distinct monomorphized sort via `family!`. Implementors expose the
//! arity as an associated const so the sort can branch on it at compile time.

pub trait Arity {
    const N: usize;
}

pub struct Binary;
impl Arity for Binary {
    const N: usize = 2;
}

pub struct Ternary;
impl Arity for Ternary {
    const N: usize = 3;
}

pub struct Base16;
impl Arity for Base16 {
    const N: usize = 16;
}

pub struct Base256;
impl Arity for Base256 {
    const N: usize = 256;
}
