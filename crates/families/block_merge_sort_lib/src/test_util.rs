//! Test helpers shared by the module tests: `Item`, ordered by `key` alone so
//! the `tag` shows which of several equal keys is which, a small generator and
//! the run and comparison helpers.

use core::cmp::Ordering;

#[derive(Clone, Copy, Debug)]
pub struct Item {
    pub key: u32,
    pub tag: u32,
}
impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool { self.key == other.key }
}
impl Eq for Item {}
impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering { self.key.cmp(&other.key) }
}

pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self, m: usize) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) as usize % m
    }
}

/// `n` items with sorted random keys below `values`, tagged from `tag0`.
pub fn sorted_run(rng: &mut Rng, n: usize, values: usize, tag0: u32) -> Vec<Item> {
    let mut keys: Vec<u32> = (0..n).map(|_| rng.next(values) as u32).collect();
    keys.sort();
    keys.into_iter().zip(tag0..).map(|(key, tag)| Item { key, tag }).collect()
}

pub fn pairs(items: &[Item]) -> Vec<(u32, u32)> { items.iter().map(|x| (x.key, x.tag)).collect() }

/// `a` then `b` merged the stable way: a sort by key that keeps equal tags in order.
pub fn merged(a: &[Item], b: &[Item]) -> Vec<(u32, u32)> {
    let mut all: Vec<Item> = a.iter().chain(b).copied().collect();
    all.sort_by_key(|x| x.key);
    pairs(&all)
}

/// `n` cells of scratch, tagged from 5000 so they are told from data.
pub fn junk(n: usize) -> Vec<Item> { (0..n).map(|i| Item { key: 7, tag: 5000 + i as u32 }).collect() }
