//! `Nodes`, the node store a `TreeSort` builds its search tree in, and the
//! `Layout`s that pack its nodes. A node is a parent, a left and a right
//! index plus a rank for the tree's own balance data, kept in one logged side
//! array: one word for an array under 2^19 elements, two under 2^32, four
//! for any length. The link operations every tree shares live in `Nodes`,
//! `attach` a leaf by key, `rotate`, `replace_child`, `unlink_min` and `place`.

use std::marker::PhantomData;

use sort_logger::SortLogger;

pub const PARENT: usize = 0;
pub const LEFT: usize = 1;
pub const RIGHT: usize = 2;
pub const RANK: usize = 3;
const WORD: usize = usize::BITS as usize;

// How the four fields of a node sit in the words of the side array. A missing
// link is the index `n`, so a layout holds `REACH` nodes at most, the largest
// `n` its link fields can name. The rank holds an AVL height, and stays 0 for a
// tree that never reads it.
pub trait Layout {
    const REACH: usize;

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
const RANK_BITS: usize = 7;

impl Packed {
    fn width(field: usize) -> usize { if field == RANK { RANK_BITS } else { PACKED_BITS } }
}

impl Layout for Packed {
    const REACH: usize = (1 << PACKED_BITS) - 1;

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

// `arr` holds the values, `meta` the nodes as `L` lays them out, and `nil` is
// the missing link. Every write to either goes through the logger.
pub struct Nodes<'a, T, U: ?Sized, L> {
    arr: &'a mut [T],
    meta: Vec<usize>,
    pub nil: usize,
    pub root: usize,
    logger: &'a mut U,
    layout: PhantomData<L>,
}

impl<'a, T: Ord + Copy, U: ?Sized + SortLogger<T>, L: Layout> Nodes<'a, T, U, L> {
    pub fn new(arr: &'a mut [T], logger: &'a mut U) -> Self {
        let n = arr.len();
        assert!(n <= L::REACH, "a layout of reach {} cannot hold {n} nodes", L::REACH);
        let meta = logger.create_aux_arr(L::words(n));
        Nodes { arr, meta, nil: n, root: n, logger, layout: PhantomData }
    }

    pub fn free(&mut self) { self.logger.free_aux_arr(&self.meta); }

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

    // Descend by key from the root and hang node `i` there as a new leaf of
    // the given rank. An equal key goes right, which is what keeps the in
    // order sequence sorted by key then by index.
    pub fn attach(&mut self, i: usize, rank: usize) {
        let (mut parent, mut side) = (self.nil, LEFT);
        let mut cur = self.root;
        while cur != self.nil {
            parent = cur;
            side = if self.logger.cmp_lt(self.arr, i, cur) { LEFT } else { RIGHT };
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

    // Move the node at `from` to the free index `to`, value and fields, and
    // point its parent and children at the new index.
    fn relocate(&mut self, from: usize, to: usize) {
        self.logger.write(self.arr, to, from);
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

    #[test]
    fn the_reaches_chain_from_a_word_to_any_length() {
        assert_eq!([Packed::REACH, Halves::REACH, Whole::REACH], [(1 << 19) - 1, u32::MAX as usize, usize::MAX]);
    }

    #[test]
    #[should_panic(expected = "cannot hold")]
    fn a_layout_refuses_an_array_past_its_reach() {
        Nodes::<usize, _, Packed>::new(&mut vec![0; Packed::REACH + 1], &mut NoOpLogger);
    }

    #[test]
    #[should_panic(expected = "node words")]
    fn a_length_whose_node_words_overflow_usize_is_refused() {
        Nodes::<(), _, Whole>::new(&mut vec![(); 1 << 62], &mut NoOpLogger);
    }
}
