//! The shapes of tree a smooth sort forest holds. `Shape` says how big a tree
//! of an order is, what its two subtrees are, and when the last two trees of a
//! forest are joined by the next element. `Leonardo`, `Perfect` and `Narayana`
//! are the three shapes, and `Forest` is the list of orders a heap is made of.

/// A family of trees, one for each order, laid out in post order: the left
/// subtree, then the right subtree, then the root.
pub trait Shape {
    /// The elements in a tree of this order.
    fn size(order: usize) -> usize;

    /// The orders of the left and the right subtree, or none for one element.
    fn children(order: usize) -> Option<(usize, usize)>;

    /// The order of the tree the next element makes of the last two trees, an
    /// older and a newer, or none when they stay apart.
    fn merged(older: usize, newer: usize) -> Option<usize>;

    /// The order of the one element tree a new element makes after `last`.
    fn single(last: Option<usize>) -> usize;
}

const fn leonardo_sizes() -> [usize; 92] {
    let mut sizes = [1usize; 92];
    let mut order = 2;
    while order < 92 {
        sizes[order] = sizes[order - 1].saturating_add(sizes[order - 2]).saturating_add(1);
        order += 1;
    }
    sizes
}

static LEONARDO_SIZES: [usize; 92] = leonardo_sizes();

/// The Leonardo trees of Dijkstra: a tree of order `k` is a root over a tree of
/// order `k - 1` and one of order `k - 2`, so its size is `L(k) = L(k - 1) +
/// L(k - 2) + 1`, with orders 0 and 1 one element each. Two trees of adjacent
/// orders are joined by the next element, which keeps the forest to a handful of
/// trees: any count is a sum of a few Leonardo numbers.
pub struct Leonardo;

impl Shape for Leonardo {
    #[inline(always)]
    fn size(order: usize) -> usize { LEONARDO_SIZES[order] }

    #[inline(always)]
    fn children(order: usize) -> Option<(usize, usize)> { (order > 1).then(|| (order - 1, order - 2)) }

    #[inline(always)]
    fn merged(older: usize, newer: usize) -> Option<usize> { (older == newer + 1).then(|| older + 1) }

    #[inline(always)]
    fn single(last: Option<usize>) -> usize { if last == Some(1) { 0 } else { 1 } }
}

/// The perfect trees of the poplar sort: a tree of order `k` is a root over two
/// trees of order `k - 1`, `2^(k + 1) - 1` elements, and two trees of one order
/// are joined by the next element. A forest is the count in skew binary, up to
/// two trees of an order, and every tree is as shallow as a tree can be.
pub struct Perfect;

impl Shape for Perfect {
    #[inline(always)]
    fn size(order: usize) -> usize { (1 << (order + 1)) - 1 }

    #[inline(always)]
    fn children(order: usize) -> Option<(usize, usize)> { (order > 0).then(|| (order - 1, order - 1)) }

    #[inline(always)]
    fn merged(older: usize, newer: usize) -> Option<usize> { (older == newer).then(|| older + 1) }

    #[inline(always)]
    fn single(_last: Option<usize>) -> usize { 0 }
}

const fn narayana_sizes() -> [usize; 140] {
    let mut sizes = [1usize; 140];
    let mut order = 3;
    while order < 140 {
        sizes[order] = sizes[order - 1].saturating_add(sizes[order - 3]).saturating_add(1);
        order += 1;
    }
    sizes
}

static NARAYANA_SIZES: [usize; 140] = narayana_sizes();

/// Trees of orders `k - 1` and `k - 3` under a root, `S(k) = S(k - 1) + S(k - 3)
/// + 1`, with orders 0, 1 and 2 one element each: the Leonardo trees with the
/// right subtree a size smaller still, so the sizes grow by 1.47 an order rather
/// than 1.62, a tree is deeper for its size and its right subtree is the small
/// one. Two trees an order and a half apart, `k - 1` and `k - 3`, are joined by
/// the next element, and a tree of order `k - 1` that waits for its partner is
/// followed by trees built up from one element in the orders it needs.
pub struct Narayana;

impl Shape for Narayana {
    #[inline(always)]
    fn size(order: usize) -> usize { NARAYANA_SIZES[order] }

    #[inline(always)]
    fn children(order: usize) -> Option<(usize, usize)> { (order > 2).then(|| (order - 1, order - 3)) }

    #[inline(always)]
    fn merged(older: usize, newer: usize) -> Option<usize> { (older == newer + 2).then(|| older + 1) }

    // The singles that build a partner for the last tree: an order 2 tree takes
    // a single of order 0, an order 3 one of order 1, and any other starts a
    // partner from order 2.
    #[inline(always)]
    fn single(last: Option<usize>) -> usize {
        match last {
            Some(2) => 0,
            Some(3) => 1,
            _ => 2,
        }
    }
}

// A perfect forest holds up to two trees of each order, the most any shape
// does, and an order is below the word's width.
const CAPACITY: usize = 2 * usize::BITS as usize;

/// The orders of the trees of a heap, oldest first, which is the order they lie
/// in the array: the last tree ends at the last element.
pub struct Forest {
    len: usize,
    orders: [usize; CAPACITY],
}

impl Default for Forest {
    fn default() -> Self { Forest { len: 0, orders: [0; CAPACITY] } }
}

impl Forest {
    pub fn orders(&self) -> &[usize] { &self.orders[..self.len] }

    pub fn push(&mut self, order: usize) {
        self.orders[self.len] = order;
        self.len += 1;
    }

    pub fn pop(&mut self) -> usize {
        assert!(self.len > 0, "no tree to take");
        self.len -= 1;
        self.orders[self.len]
    }

    /// Whether the next element would join the last two trees.
    pub fn merges_next<S: Shape>(&self) -> bool {
        matches!(self.orders(), [.., older, newer] if S::merged(*older, *newer).is_some())
    }

    /// Adds one element at the end and gives the order of the tree it ends.
    pub fn grow<S: Shape>(&mut self) -> usize {
        let order = match self.orders() {
            [.., older, newer] => S::merged(*older, *newer),
            _ => None,
        };
        match order {
            Some(joined) => {
                self.len -= 2;
                self.push(joined);
                joined
            }
            None => {
                let single = S::single(self.orders().last().copied());
                self.push(single);
                single
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forest<S: Shape>(n: usize) -> Forest {
        let mut forest = Forest::default();
        for _ in 0..n { forest.grow::<S>(); }
        forest
    }

    fn total<S: Shape>(forest: &Forest) -> usize { forest.orders().iter().map(|&order| S::size(order)).sum() }

    fn sizes_add_up<S: Shape>() {
        for order in 0..60 {
            let parts = S::children(order).map_or(0, |(left, right)| S::size(left) + S::size(right));
            assert_eq!(S::size(order), parts + 1, "order {order}");
        }
    }

    #[test]
    fn a_tree_is_its_root_and_its_two_subtrees() {
        sizes_add_up::<Leonardo>();
        sizes_add_up::<Perfect>();
        sizes_add_up::<Narayana>();
    }

    #[test]
    fn the_leonardo_numbers_and_the_perfect_sizes() {
        let leonardo: Vec<usize> = (0..8).map(Leonardo::size).collect();
        assert_eq!(leonardo, [1, 1, 3, 5, 9, 15, 25, 41]);
        let perfect: Vec<usize> = (0..6).map(Perfect::size).collect();
        assert_eq!(perfect, [1, 3, 7, 15, 31, 63]);
    }

    fn growing_makes_trees_that_hold_the_count<S: Shape>() {
        let mut forest = Forest::default();
        for n in 1..3000 {
            forest.grow::<S>();
            assert_eq!(total::<S>(&forest), n);
            assert!(forest.orders().len() <= 2 * (usize::BITS - n.leading_zeros()) as usize, "{n} elements in {} trees", forest.orders().len());
        }
    }

    #[test]
    fn growing_keeps_the_count_in_a_few_trees() {
        growing_makes_trees_that_hold_the_count::<Leonardo>();
        growing_makes_trees_that_hold_the_count::<Perfect>();
        growing_makes_trees_that_hold_the_count::<Narayana>();
    }

    #[test]
    fn the_first_leonardo_forests() {
        let orders: Vec<Vec<usize>> = (1..=9).map(|n| forest::<Leonardo>(n).orders().to_vec()).collect();
        let want: [&[usize]; 9] = [&[1], &[1, 0], &[2], &[2, 1], &[3], &[3, 1], &[3, 1, 0], &[3, 2], &[4]];
        assert_eq!(orders, want);
    }

    #[test]
    fn the_narayana_sizes_and_forests() {
        let sizes: Vec<usize> = (0..10).map(Narayana::size).collect();
        assert_eq!(sizes, [1, 1, 1, 3, 5, 7, 11, 17, 25, 37]);
        let orders: Vec<Vec<usize>> = (1..=11).map(|n| forest::<Narayana>(n).orders().to_vec()).collect();
        let want: [&[usize]; 11] = [&[2], &[2, 0], &[3], &[3, 1], &[4], &[4, 2], &[5], &[5, 2], &[5, 2, 0], &[5, 3], &[6]];
        assert_eq!(orders, want);
    }

    // The deepest tree a count of a word's width holds, the length of the path a
    // sift keeps, and the most trees its forest holds.
    #[test]
    fn the_trees_of_a_count_the_size_of_a_word_fit_the_sifts_path_and_the_forest() {
        for (name, size) in [("leonardo", Leonardo::size as fn(usize) -> usize), ("narayana", Narayana::size)] {
            let order = (0..140).find(|&order| size(order) > isize::MAX as usize).unwrap();
            assert!(order < crate::sift::DEPTH, "{name}: order {order}");
        }
        assert!(usize::BITS as usize <= crate::sift::DEPTH, "perfect: one order a bit");
    }

    #[test]
    fn the_first_perfect_forests() {
        let orders: Vec<Vec<usize>> = (1..=8).map(|n| forest::<Perfect>(n).orders().to_vec()).collect();
        let want: [&[usize]; 8] = [&[0], &[0, 0], &[1], &[1, 0], &[1, 0, 0], &[1, 1], &[2], &[2, 0]];
        assert_eq!(orders, want);
    }

    #[test]
    fn the_next_element_joins_the_last_two_trees_when_the_forest_says() {
        let mut forest = Forest::default();
        for _ in 0..500 {
            let joins = forest.merges_next::<Leonardo>();
            let before = forest.orders().len();
            forest.grow::<Leonardo>();
            assert_eq!(joins, forest.orders().len() + 1 == before);
        }
    }
}
