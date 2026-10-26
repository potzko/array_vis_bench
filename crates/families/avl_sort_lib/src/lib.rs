//! AVL sort, `AvlSort`. The input array is the node store of an AVL tree:
//! node `i` keeps its value at `arr[i]` and its links in one packed word of a
//! side array, parent, left and right index plus height. `Tree` builds the
//! tree by insertion, then `extract_min` deletes the leftmost node, moves the
//! node sitting in the next output slot into the freed slot and writes the
//! minimum there, so the sorted prefix grows from the left the way heap sort's
//! sorted suffix grows from the right. The family is declared in `avl.spec`
//! and built by `avl.avbs`.

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

pub struct AvlSort;

// A node's word holds three indices of INDEX_BITS each with the height above
// them. A missing link is the index `n`, so a sort takes at most
// 2^INDEX_BITS - 1 nodes, the `max_input` in avl.spec. An AVL tree of that
// size is under 28 levels deep, inside the seven height bits a 64 bit word
// has left.
const INDEX_BITS: u32 = 19;
const INDEX_MASK: usize = (1 << INDEX_BITS) - 1;
const PARENT: u32 = 0;
const LEFT: u32 = INDEX_BITS;
const RIGHT: u32 = 2 * INDEX_BITS;
const HEIGHT: u32 = 3 * INDEX_BITS;
pub const MAX_NODES: usize = INDEX_MASK;

impl HasTimeBounds for AvlSort {
    const WORST: Complexity = Complexity::N_LOG_N;
    const BEST: Complexity = Complexity::N_LOG_N;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
}

impl HasSpace for AvlSort {
    const SPACE: Complexity = Complexity::N1;
}

// An equal key goes right on insertion, so the in order sequence is sorted by
// key then by index, and rotations, leftmost deletion and relocation all keep
// that sequence.
impl HasStability for AvlSort {
    const STABLE: bool = true;
}

impl AvlSort {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 { return; }
        assert!(n <= MAX_NODES, "avl sort takes at most {MAX_NODES} elements, got {n}");
        let mut tree = Tree::new(arr, logger);
        for i in 0..n { tree.insert(i); }
        for k in 0..n { tree.extract_min(k); }
        tree.free();
    }
}

array_vis_bench_traits::direct_sort!(AvlSort);

// The tree over the array: `arr` holds the values, `meta` one packed word per
// node, and `nil` is the missing link. Every write to either goes through the
// logger.
struct Tree<'a, T, U: ?Sized> {
    arr: &'a mut [T],
    meta: Vec<usize>,
    nil: usize,
    root: usize,
    logger: &'a mut U,
}

impl<'a, T: Ord + Copy, U: ?Sized + SortLogger<T>> Tree<'a, T, U> {
    fn new(arr: &'a mut [T], logger: &'a mut U) -> Self {
        let n = arr.len();
        let meta = logger.create_aux_arr(n);
        Tree { arr, meta, nil: n, root: n, logger }
    }

    fn free(&mut self) { self.logger.free_aux_arr(&self.meta); }

    fn link(&self, i: usize, side: u32) -> usize { (self.meta[i] >> side) & INDEX_MASK }

    fn parent(&self, i: usize) -> usize { self.link(i, PARENT) }

    fn height(&self, i: usize) -> usize {
        if i == self.nil { 0 } else { self.meta[i] >> HEIGHT }
    }

    fn set_word(&mut self, i: usize, word: usize) {
        self.logger.write_data_u(&mut self.meta, i, word);
    }

    fn set_link(&mut self, i: usize, side: u32, to: usize) {
        self.set_word(i, (self.meta[i] & !(INDEX_MASK << side)) | (to << side));
    }

    fn set_height(&mut self, i: usize, height: usize) {
        self.set_word(i, (self.meta[i] & ((1 << HEIGHT) - 1)) | (height << HEIGHT));
    }

    fn refresh_height(&mut self, i: usize) {
        let height = 1 + self.height(self.link(i, LEFT)).max(self.height(self.link(i, RIGHT)));
        self.set_height(i, height);
    }

    // Point whichever link of `parent` led to `old` at `new`, or the root.
    fn replace_child(&mut self, parent: usize, old: usize, new: usize) {
        if parent == self.nil { self.root = new; return; }
        let side = if self.link(parent, LEFT) == old { LEFT } else { RIGHT };
        self.set_link(parent, side, new);
    }

    fn insert(&mut self, i: usize) {
        let (mut parent, mut side) = (self.nil, LEFT);
        let mut cur = self.root;
        while cur != self.nil {
            parent = cur;
            side = if self.logger.cmp_lt(self.arr, i, cur) { LEFT } else { RIGHT };
            cur = self.link(cur, side);
        }
        self.set_word(i, parent | (self.nil << LEFT) | (self.nil << RIGHT) | (1 << HEIGHT));
        if parent == self.nil { self.root = i; } else { self.set_link(parent, side, i); }
        self.retrace(parent);
    }

    // Walk up from `x` restoring heights and balance. Stops at the first node
    // whose subtree kept its height, because the heights above it are then
    // untouched: an insertion stops at its first rotation, a deletion may
    // rotate at several levels.
    fn retrace(&mut self, mut x: usize) {
        while x != self.nil {
            let parent = self.parent(x);
            let before = self.height(x);
            let (left, right) = (self.height(self.link(x, LEFT)), self.height(self.link(x, RIGHT)));
            let after = if left.abs_diff(right) > 1 {
                let top = self.rebalance(x);
                self.height(top)
            } else {
                let after = 1 + left.max(right);
                if after != before { self.set_height(x, after); }
                after
            };
            if after == before { return; }
            x = parent;
        }
    }

    // Restore balance at `x` with one rotation towards its lighter side, after
    // first rotating the heavy child when its inner grandchild is the taller,
    // the double rotation case.
    fn rebalance(&mut self, x: usize) -> usize {
        let left_heavy = self.height(self.link(x, LEFT)) > self.height(self.link(x, RIGHT));
        let (side, other) = if left_heavy { (RIGHT, LEFT) } else { (LEFT, RIGHT) };
        let heavy = self.link(x, other);
        if self.height(self.link(heavy, side)) > self.height(self.link(heavy, other)) {
            self.rotate(heavy, other, side);
        }
        self.rotate(x, side, other)
    }

    // Rotate `x` towards `side`: its child on the other side, `y`, takes its
    // place, `x` becomes the child of `y` on `side`, and the subtree `y` had
    // there moves under `x`.
    fn rotate(&mut self, x: usize, side: u32, other: u32) -> usize {
        let y = self.link(x, other);
        let moved = self.link(y, side);
        let parent = self.parent(x);
        self.set_link(x, other, moved);
        if moved != self.nil { self.set_link(moved, PARENT, x); }
        self.set_link(y, side, x);
        self.set_link(x, PARENT, y);
        self.set_link(y, PARENT, parent);
        self.replace_child(parent, x, y);
        self.refresh_height(x);
        self.refresh_height(y);
        y
    }

    // Unlink the leftmost node and return its index. It has no left child, so
    // its right subtree takes its place.
    fn remove_min(&mut self) -> usize {
        let mut least = self.root;
        while self.link(least, LEFT) != self.nil { least = self.link(least, LEFT); }
        let (parent, right) = (self.parent(least), self.link(least, RIGHT));
        self.replace_child(parent, least, right);
        if right != self.nil { self.set_link(right, PARENT, parent); }
        self.retrace(parent);
        least
    }

    // Delete the minimum and write it at `k`, the next output slot. The live
    // nodes all sit at indices from `k` up, so unless the minimum was node `k`
    // itself, node `k` first moves into the slot the deletion freed.
    fn extract_min(&mut self, k: usize) {
        let m = self.remove_min();
        if m == k { return; }
        let least = self.arr[m];
        self.relocate(k, m);
        self.logger.write_data(self.arr, k, least);
    }

    // Move the node at `from` to the free index `to`, value and word, and
    // point its parent and children at the new index.
    fn relocate(&mut self, from: usize, to: usize) {
        self.logger.write(self.arr, to, from);
        self.set_word(to, self.meta[from]);
        self.replace_child(self.parent(to), from, to);
        for side in [LEFT, RIGHT] {
            let child = self.link(to, side);
            if child != self.nil { self.set_link(child, PARENT, to); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sort_logger::NoOpLogger;

    fn assert_sorts(mut arr: Vec<usize>) {
        let mut want = arr.clone();
        want.sort();
        AvlSort::sort(&mut arr, &mut NoOpLogger);
        assert_eq!(arr, want);
    }

    // A linear congruential stream seeded by the length, so a failure replays.
    fn pseudo_random(len: usize, range: usize) -> Vec<usize> {
        let mut state = len as u64 ^ 0x9e37_79b9_7f4a_7c15;
        let step = move || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (state >> 33) as usize % range
        };
        std::iter::repeat_with(step).take(len).collect()
    }

    #[test]
    fn sorts_every_length_up_to_two_hundred() {
        for n in 0..200 { assert_sorts(pseudo_random(n, n / 3 + 1)); }
    }

    #[test]
    fn sorts_the_shaped_inputs() {
        assert_sorts((0..500).collect());
        assert_sorts((0..500).rev().collect());
        assert_sorts(vec![7; 300]);
        assert_sorts(pseudo_random(5000, 1 << 20));
    }

    // The stored height of every node is right, its children differ by at
    // most one level and each child points back at it, so the tree really is
    // an AVL tree and the bound holds. Returns the height.
    fn assert_avl<T: Ord + Copy>(tree: &Tree<T, NoOpLogger>, i: usize) -> usize {
        if i == tree.nil { return 0; }
        let (left, right) = (tree.link(i, LEFT), tree.link(i, RIGHT));
        let (left_height, right_height) = (assert_avl(tree, left), assert_avl(tree, right));
        assert!(left_height.abs_diff(right_height) <= 1, "node {i} is unbalanced");
        assert_eq!(tree.height(i), 1 + left_height.max(right_height), "node {i} height");
        for child in [left, right].into_iter().filter(|&c| c != tree.nil) {
            assert_eq!(tree.parent(child), i, "node {child} parent");
        }
        1 + left_height.max(right_height)
    }

    #[test]
    fn stays_an_avl_tree_through_every_insertion_and_extraction() {
        let mut arr = pseudo_random(300, 40);
        let n = arr.len();
        let mut logger = NoOpLogger;
        let mut tree = Tree::new(&mut arr, &mut logger);
        for i in 0..n { tree.insert(i); assert_avl(&tree, tree.root); }
        for k in 0..n { tree.extract_min(k); assert_avl(&tree, tree.root); }
        assert_eq!(tree.root, tree.nil);
    }

    // Ordered by key alone, so the index records where an element started.
    #[derive(Clone, Copy, Debug)]
    struct Keyed { key: u8, index: usize }
    impl PartialEq for Keyed { fn eq(&self, other: &Self) -> bool { self.key == other.key } }
    impl Eq for Keyed {}
    impl PartialOrd for Keyed {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
    }
    impl Ord for Keyed {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.key.cmp(&other.key) }
    }

    #[test]
    fn keeps_equal_keys_in_input_order() {
        let keys = pseudo_random(400, 5);
        let mut arr: Vec<Keyed> =
            keys.iter().enumerate().map(|(index, &key)| Keyed { key: key as u8, index }).collect();
        AvlSort::sort(&mut arr, &mut NoOpLogger);
        let in_order = |w: &[Keyed]| w[0].key < w[1].key || (w[0].key == w[1].key && w[0].index < w[1].index);
        assert!(arr.windows(2).all(in_order), "{arr:?}");
    }

    #[test]
    fn annotations_say_n_log_n_and_linear_space() {
        assert_eq!(<AvlSort as HasTimeBounds>::WORST, Complexity::N_LOG_N);
        assert!(!<AvlSort as HasSpace>::SPACE.is_in_place());
    }
}
