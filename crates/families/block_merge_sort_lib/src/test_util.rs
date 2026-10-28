//! Test helpers shared by the module tests: `Item`, ordered by `key` alone so
//! the `tag` shows which of several equal keys is which, a small generator, the
//! run and comparison helpers, and two loggers that count and collect.

use core::cmp::Ordering;

use sort_logger::{RegionKind, SortLog, SortLogger};

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

/// Counts the logged moves and comparisons, to read a growth rate off a run.
#[derive(Default)]
pub struct Tally {
    pub moves: usize,
    pub cmps: usize,
}
impl<T: Copy + Ord> SortLogger<T> for Tally {
    fn log(&mut self, event: SortLog<T>) {
        match event {
            SortLog::Swap { .. } | SortLog::WriteInArr { .. } | SortLog::WriteData { .. } | SortLog::WriteDataU { .. } => {
                self.moves += 1
            }
            SortLog::CmpInArr { .. }
            | SortLog::CmpData { .. }
            | SortLog::CmpDataU { .. }
            | SortLog::CmpAcrossArrs { .. } => self.cmps += 1,
            _ => {}
        }
    }
}

/// Keeps the marks and drops the rest.
#[derive(Default)]
pub struct Marks(pub Vec<String>);
impl<T: Copy + Ord> SortLogger<T> for Marks {
    fn log(&mut self, event: SortLog<T>) {
        if let SortLog::Mark(text) = event { self.0.push(text) }
    }
}

/// Keeps the region events as (array name, start, len, kind, held) and drops the rest.
#[derive(Default)]
pub struct Regions(pub Vec<(usize, usize, usize, RegionKind, bool)>);
impl<T: Copy + Ord> SortLogger<T> for Regions {
    fn log(&mut self, event: SortLog<T>) {
        if let SortLog::Region { name, start, len, kind, held } = event { self.0.push((name, start, len, kind, held)) }
    }
}

/// Replays the events onto copies of the arrays, so that comparing the copy of the
/// array that was sorted with the array itself shows that no move went unlogged.
/// An event finds its array by address, as the visualiser does, so a sub slice of
/// the array and the aux arrays resolve to the right copy.
pub struct Replay {
    arrays: Vec<(usize, Vec<usize>)>,
}
impl Replay {
    pub fn of(arr: &[usize]) -> Self { Replay { arrays: vec![(arr.as_ptr() as usize, arr.to_vec())] } }

    pub fn main(&self) -> &[usize] { &self.arrays[0].1 }

    fn at(&mut self, name: usize) -> (&mut Vec<usize>, usize) {
        let size = std::mem::size_of::<usize>();
        let (base, copy) = self
            .arrays
            .iter_mut()
            .filter(|(base, copy)| *base <= name && name < *base + copy.len() * size)
            .last()
            .expect("an event names an array that is not live");
        (copy, (name - *base) / size)
    }
}
impl SortLogger<usize> for Replay {
    fn log(&mut self, event: SortLog<usize>) {
        match event {
            SortLog::Swap { name, ind_a, ind_b } => {
                let (copy, off) = self.at(name);
                copy.swap(ind_a + off, ind_b + off);
            }
            SortLog::WriteInArr { name, ind_a, ind_b } => {
                let (copy, off) = self.at(name);
                copy[ind_a + off] = copy[ind_b + off];
            }
            SortLog::WriteData { name, ind, data } | SortLog::WriteDataU { name, ind, data } => {
                let (copy, off) = self.at(name);
                copy[ind + off] = data;
            }
            SortLog::CreateAuxArrT { name, length } | SortLog::CreateAuxArr { name, length } => {
                self.arrays.push((name, vec![0; length]));
            }
            SortLog::FreeAuxArr { name } => self.arrays.retain(|(base, _)| *base != name),
            _ => {}
        }
    }
}
