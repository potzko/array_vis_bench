//! Test helpers shared by the module tests: `Item`, ordered by `key` alone so
//! the `tag` shows which of several equal keys is which, a small generator,
//! a tally of comparisons and moves, a replay logger that shows no move went
//! unlogged, and a logger that keeps the cells held as stamped regions.

use core::cmp::Ordering;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

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

pub fn items(keys: &[u32]) -> Vec<Item> {
    keys.iter().zip(0u32..).map(|(&key, tag)| Item { key, tag }).collect()
}

pub fn pairs(arr: &[Item]) -> Vec<(u32, u32)> { arr.iter().map(|x| (x.key, x.tag)).collect() }

pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self, m: usize) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) as usize % m
    }
}

/// Counts the logged moves and comparisons.
#[derive(Default)]
pub struct Tally {
    pub moves: usize,
    pub cmps: usize,
}
impl<T: Copy + Ord> SortLogger<T> for Tally {
    fn log(&mut self, event: SortLog<T>) {
        match event {
            SortLog::Swap { .. } | SortLog::WriteInArr { .. } | SortLog::WriteData { .. } => self.moves += 1,
            SortLog::CmpInArr { .. } | SortLog::CmpData { .. } => self.cmps += 1,
            _ => {}
        }
    }
}

/// Replays the events onto a copy of the array, so that comparing the copy with
/// the array itself shows that no move went unlogged. An event finds its array
/// by address, so a sub slice resolves to the right offset.
pub struct Replay {
    base: usize,
    copy: Vec<usize>,
}
impl Replay {
    pub fn of(arr: &[usize]) -> Self { Replay { base: arr.as_ptr() as usize, copy: arr.to_vec() } }

    pub fn main(&self) -> &[usize] { &self.copy }

    fn offset(&self, name: usize) -> usize { (name - self.base) / std::mem::size_of::<usize>() }
}
impl SortLogger<usize> for Replay {
    fn log(&mut self, event: SortLog<usize>) {
        match event {
            SortLog::Swap { name, ind_a, ind_b } => {
                let off = self.offset(name);
                self.copy.swap(ind_a + off, ind_b + off);
            }
            SortLog::WriteInArr { name, ind_a, ind_b } => {
                let off = self.offset(name);
                self.copy[ind_a + off] = self.copy[ind_b + off];
            }
            SortLog::WriteData { name, ind, data } => {
                let off = self.offset(name);
                self.copy[ind + off] = data;
            }
            _ => {}
        }
    }
}

/// Keeps the cells currently held as stamped regions, shared between the logger
/// the sort writes to and the test that reads it back mid sort.
#[derive(Clone, Default)]
pub struct Held(Rc<RefCell<HashSet<usize>>>);
impl Held {
    pub fn cells(&self) -> Vec<usize> { self.0.borrow().iter().copied().collect() }
}
impl<T: Copy + Ord> SortLogger<T> for Held {
    fn log(&mut self, event: SortLog<T>) {
        if let SortLog::Region { start, len, kind: RegionKind::Stamped, held, .. } = event {
            assert_eq!(len, 1, "a stamped region is one cell");
            let changed = if held { self.0.borrow_mut().insert(start) } else { self.0.borrow_mut().remove(&start) };
            assert!(changed, "cell {start} {} twice", if held { "held" } else { "released" });
        }
    }
}
