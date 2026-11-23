//! Test helpers shared by the module tests: `Item`, ordered by `key` alone so
//! the `tag` shows which of several equal keys is which, a small generator, the
//! pair view of an array, the expected outcome of a partition, a tally of the
//! logged moves and comparisons and `Needy`, a partition that insists on its
//! scratch.

use core::cmp::Ordering;

use crate::RotationPartition;
use array_vis_bench_traits::role::{Rotation, StablePartition};
use array_vis_bench_traits::Complexity;
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

pub fn items(keys: &[u32]) -> Vec<Item> {
    keys.iter().zip(0u32..).map(|(&key, tag)| Item { key, tag }).collect()
}

pub fn item(key: u32) -> Item { Item { key, tag: 0 } }

pub fn pairs(arr: &[Item]) -> Vec<(u32, u32)> { arr.iter().map(|x| (x.key, x.tag)).collect() }

pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self, m: usize) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) as usize % m
    }
}

/// Partitions by rotation, wants a scratch of its own as long as the region and
/// fails when it is handed a shorter one.
pub struct Needy;
impl StablePartition for Needy {
    const NAME: &'static str = "needy";
    const MOVES: Complexity = Complexity::N_LOG_N;

    fn scratch_size(n: usize) -> usize { n }

    fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        assert!(scratch.len() >= hi - lo, "a scratch of {} for a region of {}", scratch.len(), hi - lo);
        RotationPartition::partition::<R, T, U>(arr, lo, hi, pivot, scratch, log)
    }

    fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        assert!(scratch.len() >= hi - lo, "a scratch of {} for a region of {}", scratch.len(), hi - lo);
        RotationPartition::partition_at_most::<R, T, U>(arr, lo, hi, pivot, scratch, log)
    }
}

/// The expected outcome of a partition, spelled out: the elements the predicate
/// picks in order, then the rest in order.
pub fn reference(arr: &[Item], picked: impl Fn(u32) -> bool) -> Vec<(u32, u32)> {
    let first = arr.iter().filter(|x| picked(x.key));
    let rest = arr.iter().filter(|x| !picked(x.key));
    first.chain(rest).map(|x| (x.key, x.tag)).collect()
}

/// The expected three way split: below, equal and above, each in order.
pub fn three_way(arr: &[Item], pivot: u32) -> Vec<(u32, u32)> {
    let below = arr.iter().filter(|x| x.key < pivot);
    let equal = arr.iter().filter(|x| x.key == pivot);
    let above = arr.iter().filter(|x| x.key > pivot);
    below.chain(equal).chain(above).map(|x| (x.key, x.tag)).collect()
}

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
            SortLog::CmpInArr { .. } | SortLog::CmpData { .. } | SortLog::CmpDataU { .. } | SortLog::CmpAcrossArrs { .. } => {
                self.cmps += 1
            }
            _ => {}
        }
    }
}

/// Replays the events onto copies of the arrays, so that comparing the copy of
/// the array that was partitioned with the array itself shows that no move went
/// unlogged. An event finds its array by address, as the visualiser does, so a
/// sub slice of the array, which is what a rotation is handed, resolves to the
/// right copy and offset.
pub struct Replay {
    arrays: Vec<(usize, Vec<usize>)>,
}
impl Replay {
    pub fn of(arr: &[usize]) -> Self { Replay { arrays: vec![(arr.as_ptr() as usize, arr.to_vec())] } }

    pub fn main(&self) -> &[usize] { &self.arrays[0].1 }

    // The copy an address falls in, and the offset of the address in it.
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
