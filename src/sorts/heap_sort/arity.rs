//! Heap arity — number of children per heap node.
//!
//! Used as a type-level parameter for `HeapSort<A>` so each arity produces a
//! distinct monomorphized sort via `family!`. Implementors expose the
//! arity as an associated const so the sort can branch on it at compile time.

pub trait Arity {
    const N: usize;
}

pub struct Binary;
combo_codegen::component!(Arity, Binary, "binary");
impl Arity for Binary {
    const N: usize = 2;
}

pub struct Ternary;
combo_codegen::component!(Arity, Ternary, "ternary");
impl Arity for Ternary {
    const N: usize = 3;
}

pub struct Base16;
combo_codegen::component!(Arity, Base16, "16-ary");
impl Arity for Base16 {
    const N: usize = 16;
}

pub struct Base256;
combo_codegen::component!(Arity, Base256, "256-ary");
impl Arity for Base256 {
    const N: usize = 256;
}
