//! `Nodes`, the node store a `TreeSort` builds its search tree in, the
//! `Layout`s that pack its nodes, and `NodeKeys`, what it orders them by. A
//! node is a parent, a left and a right index plus a rank for the tree's own
//! balance data, kept in one logged side array: one word for an array under
//! 2^19 elements, two under 2^32, four for any length. The store holds indices
//! alone, and its caller hands it the keys. The link operations every tree
//! shares live in `Nodes`, `attach` a leaf by key, `rotate`, `replace_child`,
//! `unlink_min` and `place`.

use std::marker::PhantomData;

use sort_logger::SortLogger;

pub const PARENT: usize = 0;
pub const LEFT: usize = 1;
pub const RIGHT: usize = 2;
pub const RANK: usize = 3;
const WORD: usize = usize::BITS as usize;

// How the four fields of a node sit in the words of the side array. A missing
// link is the index `n`, so a layout holds `REACH` nodes at most, the largest
// `n` its link fields can name. The rank is the tree's own balance data,
// `RANK_BITS` wide, and stays 0 for a tree that never reads it.
pub trait Layout {
    const REACH: usize;
    const RANK_BITS: usize;

    // Words of side array for `n` nodes.
    fn words(n: usize) -> usize;

    fn get(meta: &[usize], i: usize, field: usize) -> usize;

    fn set<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        meta: &mut [usize],
        logger: &mut U,
        i: usize,
        field: usize,
        value: usize,
    );

    // A whole node, parent then left then right then rank, and a node copied
    // whole. A layout that keeps a node in one word does each in one write.
    fn put<T: Ord + Copy, U: ?Sized + SortLogger<T>>(meta: &mut [usize], logger: &mut U, i: usize, node: [usize; 4]) {
        for (field, value) in node.into_iter().enumerate() { Self::set(meta, logger, i, field, value); }
    }

    fn copy<T: Ord + Copy, U: ?Sized + SortLogger<T>>(meta: &mut [usize], logger: &mut U, from: usize, to: usize) {
        for field in [PARENT, LEFT, RIGHT, RANK] { Self::set(meta, logger, to, field, Self::get(meta, from, field)); }
    }
}

// Three links of 19 bits, which name every index below 2^19 and the missing
// link, and 7 bits of rank fill one word. A node is written and copied as that
// one word, a single logged write each.
pub struct Packed;
const PACKED_BITS: usize = 19;
const PACKED_RANK_BITS: usize = 7;

impl Packed {
    fn width(field: usize) -> usize { if field == RANK { PACKED_RANK_BITS } else { PACKED_BITS } }
}

impl Layout for Packed {
    const REACH: usize = (1 << PACKED_BITS) - 1;
    const RANK_BITS: usize = PACKED_RANK_BITS;

    fn words(n: usize) -> usize { n }

    fn get(meta: &[usize], i: usize, field: usize) -> usize {
        meta[i] >> (field * PACKED_BITS) & (usize::MAX >> (WORD - Self::width(field)))
    }

    fn set<T: Ord + Copy, U: ?Sized + SortLogger<T>>(meta: &mut [usize], logger: &mut U, i: usize, field: usize, value: usize) {
        let mask = usize::MAX >> (WORD - Self::width(field));
        debug_assert!(value & !mask == 0, "{value} does not fit field {field}");
        let word = meta[i] & !(mask << (field * PACKED_BITS)) | value << (field * PACKED_BITS);
        logger.write_data_u(meta, i, word);
    }

    fn put<T: Ord + Copy, U: ?Sized + SortLogger<T>>(meta: &mut [usize], logger: &mut U, i: usize, node: [usize; 4]) {
        let word = node.iter().zip(0..).fold(0, |word, (value, field)| word | value << (field * PACKED_BITS));
        logger.write_data_u(meta, i, word);
    }

    fn copy<T: Ord + Copy, U: ?Sized + SortLogger<T>>(meta: &mut [usize], logger: &mut U, from: usize, to: usize) {
        let word = meta[from];
        logger.write_data_u(meta, to, word);
    }
}

// Every field BITS wide and WORDS words to a node, no field straddling a word.
// `Halves` takes two words of 32 bit fields for an array under 2^32 elements,
// `Whole` a word for each field, so any length a machine can hold has a layout.
pub struct Fields<const WORDS: usize, const BITS: usize>;
pub type Halves = Fields<2, 32>;
pub type Whole = Fields<4, 64>;

impl<const WORDS: usize, const BITS: usize> Fields<WORDS, BITS> {
    const MASK: usize = usize::MAX >> (WORD - BITS);

    // The word of the field and the bit it starts at there.
    fn locate(i: usize, field: usize) -> (usize, usize) {
        (WORDS * i + field / (WORD / BITS), field % (WORD / BITS) * BITS)
    }
}

impl<const WORDS: usize, const BITS: usize> Layout for Fields<WORDS, BITS> {
    const REACH: usize = Self::MASK;
    const RANK_BITS: usize = BITS;

    fn words(n: usize) -> usize { n.checked_mul(WORDS).expect("the node words of the array overflow usize") }

    fn get(meta: &[usize], i: usize, field: usize) -> usize {
        let (word, shift) = Self::locate(i, field);
        meta[word] >> shift & Self::MASK
    }

    fn set<T: Ord + Copy, U: ?Sized + SortLogger<T>>(meta: &mut [usize], logger: &mut U, i: usize, field: usize, value: usize) {
        let (word, shift) = Self::locate(i, field);
        debug_assert!(value & !Self::MASK == 0, "{value} does not fit {BITS} bits");
        let kept = meta[word] & !(Self::MASK << shift);
        logger.write_data_u(meta, word, kept | value << shift);
    }
}

/// What the tree orders its nodes by: whether node `a`'s key is less than node
/// `b`'s, compared through the logger. A slice is its own keys, node `i`
/// keyed by its `i`th value.
pub trait NodeKeys<T: Ord + Copy> {
    fn less<U: ?Sized + SortLogger<T>>(&self, logger: &mut U, a: usize, b: usize) -> bool;
}

impl<T: Ord + Copy> NodeKeys<T> for [T] {
    fn less<U: ?Sized + SortLogger<T>>(&self, logger: &mut U, a: usize, b: usize) -> bool { logger.cmp_lt(self, a, b) }
}

// `meta` holds the nodes as `L` lays them out, and `nil` is the missing link.
// Every write to it goes through the logger. The keys are the caller's.
pub struct Nodes<'a, T, U: ?Sized, L> {
    meta: &'a mut [usize],
    pub nil: usize,
    pub root: usize,
    logger: &'a mut U,
    types: PhantomData<(T, L)>,
}

impl<'a, T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout> Nodes<'a, T, U, L> {
    // A tree of up to `n` nodes living in `meta`, at least `L::words(n)` words,
    // which the caller creates through the logger and frees once the tree is
    // done with.
    pub fn over(n: usize, meta: &'a mut [usize], logger: &'a mut U) -> Self {
        assert!(n <= L::REACH, "a layout of reach {} cannot hold {n} nodes", L::REACH);
        assert!(meta.len() >= L::words(n), "{} words cannot hold the nodes of {n}", meta.len());
        Nodes { meta, nil: n, root: n, logger, types: PhantomData }
    }

    // The logger, for a caller that logs its own work between tree operations.
    pub fn logger(&mut self) -> &mut U { self.logger }

    pub fn link(&self, i: usize, side: usize) -> usize { L::get(&self.meta, i, side) }

    pub fn parent(&self, i: usize) -> usize { self.link(i, PARENT) }

    pub fn rank(&self, i: usize) -> usize {
        if i == self.nil { 0 } else { L::get(&self.meta, i, RANK) }
    }

    pub fn set_link(&mut self, i: usize, side: usize, to: usize) {
        L::set::<T, U>(&mut self.meta, self.logger, i, side, to);
    }

    pub fn set_rank(&mut self, i: usize, rank: usize) { L::set::<T, U>(&mut self.meta, self.logger, i, RANK, rank); }

    // Point whichever link of `parent` led to `old` at `new`, or the root.
    pub fn replace_child(&mut self, parent: usize, old: usize, new: usize) {
        if parent == self.nil { self.root = new; return; }
        let side = if self.link(parent, LEFT) == old { LEFT } else { RIGHT };
        self.set_link(parent, side, new);
    }

    // Descend by `keys` from the root and hang node `i` there as a new leaf of
    // the given rank. An equal key goes right, which is what keeps the in
    // order sequence sorted by key then by index.
    pub fn attach<K: NodeKeys<T> + ?Sized>(&mut self, i: usize, rank: usize, keys: &K) {
        let (mut parent, mut side) = (self.nil, LEFT);
        let mut cur = self.root;
        while cur != self.nil {
            parent = cur;
            side = if keys.less(self.logger, i, cur) { LEFT } else { RIGHT };
            cur = self.link(cur, side);
        }
        L::put::<T, U>(&mut self.meta, self.logger, i, [parent, self.nil, self.nil, rank]);
        if parent == self.nil { self.root = i; } else { self.set_link(parent, side, i); }
    }

    // Rotate `x` towards `side`: its child on the other side, `y`, takes its
    // place, `x` becomes the child of `y` on `side`, and the subtree `y` had
    // there moves under `x`. Returns `y`. Ranks are left to the caller.
    pub fn rotate(&mut self, x: usize, side: usize, other: usize) -> usize {
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

    // Write the value of the unlinked node `m` at `k` of `arr`, the next output
    // slot, for a tree whose node `i` is the value `arr[i]`. The live nodes all
    // sit at indices from `k` up, so unless the minimum was node `k` itself,
    // node `k` first moves into the slot the deletion freed.
    pub fn place(&mut self, arr: &mut [T], m: usize, k: usize) {
        if m == k { return; }
        let least = arr[m];
        self.relocate(arr, k, m);
        self.logger.write_data(arr, k, least);
    }

    // Move the node at `from` to the free index `to`, value and fields, and
    // point its parent and children at the new index.
    fn relocate(&mut self, arr: &mut [T], from: usize, to: usize) {
        self.logger.write(arr, to, from);
        L::copy::<T, U>(&mut self.meta, self.logger, from, to);
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

    fn node<L: Layout>(meta: &[usize], i: usize) -> [usize; 4] { [PARENT, LEFT, RIGHT, RANK].map(|field| L::get(meta, i, field)) }

    // A node written whole, one field changed and the node copied reads back
    // as written, whatever the neighbours hold, with links as big as the
    // layout reaches.
    fn assert_round_trips<L: Layout>(n: usize, top: usize) {
        let (mut meta, mut logger) = (vec![0; L::words(n)], NoOpLogger);
        L::put::<usize, _>(&mut meta, &mut logger, 3, [top, top / 3, 5, 90]);
        assert_eq!(node::<L>(&meta, 3), [top, top / 3, 5, 90]);
        L::set::<usize, _>(&mut meta, &mut logger, 3, LEFT, top);
        L::copy::<usize, _>(&mut meta, &mut logger, 3, n - 1);
        assert_eq!(node::<L>(&meta, n - 1), [top, top, 5, 90]);
        for untouched in [0, 2, 4, n - 2] { assert_eq!(node::<L>(&meta, untouched), [0; 4], "node {untouched}"); }
    }

    #[test]
    fn packed_holds_every_index_up_to_its_reach() { assert_round_trips::<Packed>(Packed::REACH, Packed::REACH); }

    #[test]
    fn halves_hold_every_index_up_to_a_u32() { assert_round_trips::<Halves>(1 << 20, Halves::REACH); }

    #[test]
    fn whole_holds_a_word_in_every_field() { assert_round_trips::<Whole>(1 << 20, usize::MAX); }

    // The widest rank a layout claims to hold sits in the word beside links as
    // wide as it reaches, and reads back whole.
    fn assert_holds_its_widest_rank<L: Layout>(n: usize) {
        let widest = usize::MAX >> (usize::BITS as usize - L::RANK_BITS);
        let (mut meta, mut logger) = (vec![0; L::words(n)], NoOpLogger);
        L::put::<usize, _>(&mut meta, &mut logger, 1, [L::REACH, L::REACH, L::REACH, widest]);
        assert_eq!(node::<L>(&meta, 1), [L::REACH, L::REACH, L::REACH, widest]);
        L::set::<usize, _>(&mut meta, &mut logger, 1, RANK, 0);
        assert_eq!(node::<L>(&meta, 1), [L::REACH, L::REACH, L::REACH, 0]);
    }

    #[test]
    fn every_layout_holds_the_widest_rank_it_claims() {
        assert_eq!([Packed::RANK_BITS, Halves::RANK_BITS, Whole::RANK_BITS], [7, 32, 64]);
        assert_holds_its_widest_rank::<Packed>(4);
        assert_holds_its_widest_rank::<Halves>(4);
        assert_holds_its_widest_rank::<Whole>(4);
    }

    #[test]
    fn the_reaches_chain_from_a_word_to_any_length() {
        assert_eq!([Packed::REACH, Halves::REACH, Whole::REACH], [(1 << 19) - 1, u32::MAX as usize, usize::MAX]);
    }

    // A priority queue's round, which tree sort never plays: the least node
    // leaves, takes a new key and goes back in, so the keys come out merged.
    #[test]
    fn a_node_unlinked_as_the_least_goes_back_in_under_a_new_key() {
        let mut keys = vec![5usize, 1, 4, 2, 3];
        let mut meta = vec![0; Packed::words(keys.len())];
        let mut logger = NoOpLogger;
        let mut nodes = Nodes::<_, _, Packed>::over(keys.len(), &mut meta, &mut logger);
        (0..5).for_each(|i| nodes.attach(i, 0, &keys[..]));
        let mut out = Vec::new();
        for _ in 0..10 {
            let (least, _) = nodes.unlink_min();
            out.push(keys[least]);
            keys[least] += 10;
            nodes.attach(least, 0, &keys[..]);
        }
        assert_eq!(out, [1, 2, 3, 4, 5, 11, 12, 13, 14, 15]);
    }

    // Orders the nodes by the keys of `0` backwards, so the least is the largest.
    struct Backwards(Vec<usize>);
    impl NodeKeys<usize> for Backwards {
        fn less<U: ?Sized + SortLogger<usize>>(&self, logger: &mut U, a: usize, b: usize) -> bool { logger.cmp_gt(&self.0, a, b) }
    }

    #[test]
    fn the_tree_orders_its_nodes_by_the_keys_it_is_handed() {
        let keys = Backwards(vec![5, 1, 4, 2, 3]);
        let mut meta = vec![0; Packed::words(5)];
        let mut logger = NoOpLogger;
        let mut nodes = Nodes::<usize, _, Packed>::over(5, &mut meta, &mut logger);
        (0..5).for_each(|i| nodes.attach(i, 0, &keys));
        let out: Vec<usize> = (0..5).map(|_| keys.0[nodes.unlink_min().0]).collect();
        assert_eq!(out, [5, 4, 3, 2, 1]);
    }

    #[test]
    #[should_panic(expected = "cannot hold")]
    fn a_layout_refuses_an_array_past_its_reach() {
        Nodes::<usize, _, Packed>::over(Packed::REACH + 1, &mut [], &mut NoOpLogger);
    }

    #[test]
    #[should_panic(expected = "node words")]
    fn a_length_whose_node_words_overflow_usize_is_refused() {
        Nodes::<(), _, Whole>::over(1 << 62, &mut [], &mut NoOpLogger);
    }
}
