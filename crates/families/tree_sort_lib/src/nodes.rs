//! `Nodes`, the packed node store a `TreeSort` builds its search tree in. It
//! borrows the array and the logger and keeps one word per node: parent, left
//! and right index plus a rank field for the tree's own balance data. The
//! link operations every tree shares live here, `attach` a leaf by key,
//! `rotate`, `replace_child`, `unlink_min` and `place`; a tree's balance rule
//! calls back into them from its `Tree` impl.

use sort_logger::SortLogger;

// A node's word holds three indices of INDEX_BITS each with the rank above
// them. A missing link is the index `n`, so a sort takes at most
// 2^INDEX_BITS - 1 nodes, the `max_input` in tree.spec. The seven rank bits a
// 64 bit word has left hold an AVL height, which stays under 28 at that size.
const INDEX_BITS: u32 = 19;
const INDEX_MASK: usize = (1 << INDEX_BITS) - 1;
const PARENT: u32 = 0;
pub const LEFT: u32 = INDEX_BITS;
pub const RIGHT: u32 = 2 * INDEX_BITS;
const RANK: u32 = 3 * INDEX_BITS;
pub const MAX_NODES: usize = INDEX_MASK;

// `arr` holds the values, `meta` one packed word per node, and `nil` is the
// missing link. Every write to either goes through the logger.
pub struct Nodes<'a, T, U: ?Sized> {
    arr: &'a mut [T],
    meta: Vec<usize>,
    pub nil: usize,
    pub root: usize,
    logger: &'a mut U,
}

impl<'a, T: Ord + Copy, U: ?Sized + SortLogger<T>> Nodes<'a, T, U> {
    pub fn new(arr: &'a mut [T], logger: &'a mut U) -> Self {
        let n = arr.len();
        let meta = logger.create_aux_arr(n);
        Nodes { arr, meta, nil: n, root: n, logger }
    }

    pub fn free(&mut self) { self.logger.free_aux_arr(&self.meta); }

    pub fn link(&self, i: usize, side: u32) -> usize { (self.meta[i] >> side) & INDEX_MASK }

    pub fn parent(&self, i: usize) -> usize { self.link(i, PARENT) }

    pub fn rank(&self, i: usize) -> usize {
        if i == self.nil { 0 } else { self.meta[i] >> RANK }
    }

    fn set_word(&mut self, i: usize, word: usize) {
        self.logger.write_data_u(&mut self.meta, i, word);
    }

    pub fn set_link(&mut self, i: usize, side: u32, to: usize) {
        self.set_word(i, (self.meta[i] & !(INDEX_MASK << side)) | (to << side));
    }

    pub fn set_rank(&mut self, i: usize, rank: usize) {
        self.set_word(i, (self.meta[i] & ((1 << RANK) - 1)) | (rank << RANK));
    }

    // Point whichever link of `parent` led to `old` at `new`, or the root.
    pub fn replace_child(&mut self, parent: usize, old: usize, new: usize) {
        if parent == self.nil { self.root = new; return; }
        let side = if self.link(parent, LEFT) == old { LEFT } else { RIGHT };
        self.set_link(parent, side, new);
    }

    // Descend by key from the root and hang node `i` there as a new leaf of
    // rank 1, the height of a leaf. An equal key goes right, which is what
    // keeps the in order sequence sorted by key then by index.
    pub fn attach(&mut self, i: usize) {
        let (mut parent, mut side) = (self.nil, LEFT);
        let mut cur = self.root;
        while cur != self.nil {
            parent = cur;
            side = if self.logger.cmp_lt(self.arr, i, cur) { LEFT } else { RIGHT };
            cur = self.link(cur, side);
        }
        self.set_word(i, parent | (self.nil << LEFT) | (self.nil << RIGHT) | (1 << RANK));
        if parent == self.nil { self.root = i; } else { self.set_link(parent, side, i); }
    }

    // Rotate `x` towards `side`: its child on the other side, `y`, takes its
    // place, `x` becomes the child of `y` on `side`, and the subtree `y` had
    // there moves under `x`. Returns `y`. Ranks are left to the caller.
    pub fn rotate(&mut self, x: usize, side: u32, other: u32) -> usize {
        let y = self.link(x, other);
        let moved = self.link(y, side);
        let parent = self.parent(x);
        self.set_link(x, other, moved);
        if moved != self.nil { self.set_link(moved, PARENT, x); }
        self.set_link(y, side, x);
        self.set_link(x, PARENT, y);
        self.set_link(y, PARENT, parent);
        self.replace_child(parent, x, y);
        y
    }

    // Unlink the leftmost node and return it with its parent. It has no left
    // child, so its right subtree takes its place.
    pub fn unlink_min(&mut self) -> (usize, usize) {
        let mut least = self.root;
        while self.link(least, LEFT) != self.nil { least = self.link(least, LEFT); }
        let (parent, right) = (self.parent(least), self.link(least, RIGHT));
        self.replace_child(parent, least, right);
        if right != self.nil { self.set_link(right, PARENT, parent); }
        (least, parent)
    }

    // Write the value of the unlinked node `m` at `k`, the next output slot.
    // The live nodes all sit at indices from `k` up, so unless the minimum
    // was node `k` itself, node `k` first moves into the slot the deletion
    // freed.
    pub fn place(&mut self, m: usize, k: usize) {
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
