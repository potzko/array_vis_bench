//! Heap direction — pairs a [`Compare`] with a [`Layout`].
//!
//! All 4 (compare, layout) combinations exist as concrete types so future
//! sorts can use whichever they need. For HEAP SORT only 2 produce
//! ascending output (`MinReverse`, `MaxForward`); those carry the
//! `HeapDirection` `component!` marker so the family! call
//! auto-registers them. The other two (`MinForward`, `MaxReverse`) sort
//! to descending order — they exist as types but aren't wired into the
//! ascending heap sort.

use super::compare::{Compare, Max, Min};
use super::layout::{Forward, Layout, Reverse};

pub trait Direction {
    type Compare: Compare;
    type Layout: Layout;
}

pub struct MinForward;
impl Direction for MinForward {
    type Compare = Min;
    type Layout = Forward;
}

pub struct MinReverse;
combo_codegen::component!(HeapDirection, MinReverse, "min reverse");
impl Direction for MinReverse {
    type Compare = Min;
    type Layout = Reverse;
}

pub struct MaxForward;
combo_codegen::component!(HeapDirection, MaxForward, "max forward");
impl Direction for MaxForward {
    type Compare = Max;
    type Layout = Forward;
}

pub struct MaxReverse;
impl Direction for MaxReverse {
    type Compare = Max;
    type Layout = Reverse;
}
