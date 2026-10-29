//! Tree sort, `TreeSort<B>`: the array as the node store of a binary search
//! tree over a tree axis, a node packed in one word up to 2^19 elements and
//! in wider layouts beyond, so no length is too long. `nodes.rs` is the store
//! and its layouts, `tree_sort.rs` the driver and the `Tree` role, `avl.rs`
//! and `naive.rs` its two fillers. The family is declared in `tree.spec` and
//! built by `tree.avbs`.

pub mod avl;
pub mod naive;
pub mod nodes;
pub mod tree_sort;

pub use avl::Avl;
pub use naive::Naive;
pub use nodes::{Fields, Halves, Layout, Nodes, Packed, Whole};
pub use tree_sort::{Tree, TreeSort};

#[cfg(test)]
pub(crate) mod test_input {
    // A linear congruential stream seeded by the length, so a failure replays.
    pub fn pseudo_random(len: usize, range: usize) -> Vec<usize> {
        let mut state = len as u64 ^ 0x9e37_79b9_7f4a_7c15;
        let step = move || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (state >> 33) as usize % range
        };
        std::iter::repeat_with(step).take(len).collect()
    }
}
