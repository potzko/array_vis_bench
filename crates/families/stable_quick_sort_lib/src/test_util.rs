//! Test helpers shared by the module tests: `Item`, ordered by `key` alone so
//! the `tag` shows which of several equal keys is which, a small generator,
//! a tally of comparisons and moves, and a replay logger over every array the
//! sort announces, which shows no move went unlogged.

use core::cmp::Ordering;

use sort_logger::{SortLog, SortLogger};

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

pub fn items(keys: &[u32]) -> Vec<Item> {
    keys.iter().zip(0u32..).map(|(&key, tag)| Item { key, tag }).collect()
}

pub fn pairs(items: &[Item]) -> Vec<(u32, u32)> { items.iter().map(|x| (x.key, x.tag)).collect() }

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

    // An array addressed whole is found by its address; a sub slice by the
    // range of the array holding it, in words, which only the main array of
    // elements is ever cut into.
    fn at(&mut self, name: usize) -> (&mut Vec<usize>, usize) {
        let size = std::mem::size_of::<usize>();
        let whole = self.arrays.iter().position(|(base, _)| *base == name);
        let (base, copy) = match whole {
            Some(i) => &mut self.arrays[i],
            None => self
                .arrays
                .iter_mut()
                .filter(|(base, copy)| *base <= name && name < *base + copy.len() * size)
                .last()
                .expect("an event names an array that is not live"),
        };
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
