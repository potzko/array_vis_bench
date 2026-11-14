//! The `Marks` fillers, where a flag per buffer cell is kept. `ByteMarks` is an
//! aux array of a byte per mark and `BitMarks` one of a bit per mark, packed
//! into words. `InArrayMarks` allocates nothing: a mark is two distinct
//! elements carved from the array, in order while it is clear and swapped
//! while it is set, so reading it is a comparison and writing it a swap.

use array_vis_bench_traits::role::Marks;
use sort_logger::SortLogger;

/// A byte per mark in an aux array.
pub struct ByteMarks;
impl Marks for ByteMarks {
    const NAME: &'static str = "bytes";
    const ALLOCATED_BITS: usize = 8;
    const CARVED: usize = 0;
    type Store = Vec<u8>;

    fn create<T: Ord + Copy, U: ?Sized + SortLogger<T>>(cells: usize, log: &mut U) -> Vec<u8> { log.create_aux_arr_u8(cells) }

    fn free<T: Ord + Copy, U: ?Sized + SortLogger<T>>(store: &Vec<u8>, log: &mut U) { log.free_aux_arr_u8(store) }

    fn is_marked<T: Ord + Copy, U: ?Sized + SortLogger<T>>(store: &Vec<u8>, _carved: &[T], i: usize, _log: &mut U) -> bool { store[i] != 0 }

    fn mark<T: Ord + Copy, U: ?Sized + SortLogger<T>>(store: &mut Vec<u8>, _carved: &mut [T], i: usize, log: &mut U) { log.write_data_u8(store, i, 1) }

    fn unmark<T: Ord + Copy, U: ?Sized + SortLogger<T>>(store: &mut Vec<u8>, _carved: &mut [T], i: usize, log: &mut U) { log.write_data_u8(store, i, 0) }
}

const WORD: usize = usize::BITS as usize;

/// A bit per mark, packed into an aux array of words: the logarithm of marks
/// is a constant number of words.
pub struct BitMarks;
impl BitMarks {
    fn flip<T: Ord + Copy, U: ?Sized + SortLogger<T>>(store: &mut [usize], i: usize, log: &mut U) {
        log.write_data_u(store, i / WORD, store[i / WORD] ^ 1 << (i % WORD))
    }
}
impl Marks for BitMarks {
    const NAME: &'static str = "bits";
    const ALLOCATED_BITS: usize = 1;
    const CARVED: usize = 0;
    type Store = Vec<usize>;

    fn create<T: Ord + Copy, U: ?Sized + SortLogger<T>>(cells: usize, log: &mut U) -> Vec<usize> { log.create_aux_arr(cells.div_ceil(WORD)) }

    fn free<T: Ord + Copy, U: ?Sized + SortLogger<T>>(store: &Vec<usize>, log: &mut U) { log.free_aux_arr(store) }

    fn is_marked<T: Ord + Copy, U: ?Sized + SortLogger<T>>(store: &Vec<usize>, _carved: &[T], i: usize, _log: &mut U) -> bool {
        store[i / WORD] >> (i % WORD) & 1 == 1
    }

    fn mark<T: Ord + Copy, U: ?Sized + SortLogger<T>>(store: &mut Vec<usize>, _carved: &mut [T], i: usize, log: &mut U) { Self::flip(store, i, log) }

    fn unmark<T: Ord + Copy, U: ?Sized + SortLogger<T>>(store: &mut Vec<usize>, _carved: &mut [T], i: usize, log: &mut U) { Self::flip(store, i, log) }
}

/// Two distinct elements of the array per mark, the cells `2i` and `2i + 1` of
/// the carved region: ascending is clear, descending is set. The elements only
/// ever trade places with their partner, so the region is sorted again once
/// every mark is clear.
pub struct InArrayMarks;
impl Marks for InArrayMarks {
    const NAME: &'static str = "in array";
    const ALLOCATED_BITS: usize = 0;
    const CARVED: usize = 2;
    type Store = ();

    fn create<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_cells: usize, _log: &mut U) {}

    fn free<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_store: &(), _log: &mut U) {}

    fn is_marked<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_store: &(), carved: &[T], i: usize, log: &mut U) -> bool {
        log.cmp_lt(carved, 2 * i + 1, 2 * i)
    }

    fn mark<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_store: &mut (), carved: &mut [T], i: usize, log: &mut U) { log.swap(carved, 2 * i, 2 * i + 1) }

    fn unmark<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_store: &mut (), carved: &mut [T], i: usize, log: &mut U) { log.swap(carved, 2 * i, 2 * i + 1) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sort_logger::NoOpLogger;

    // Marks every third cell and the last, reads all of them back, clears them
    // and reads again: a mark is seen where it was set and nowhere else.
    fn marks_hold<M: Marks>(cells: usize) {
        let log = &mut NoOpLogger;
        let mut carved: Vec<usize> = (0..cells * M::CARVED).collect();
        let sorted = carved.clone();
        let mut store = M::create::<usize, _>(cells, log);
        let chosen = |i: usize| i % 3 == 0 || i + 1 == cells;
        let read = |store: &M::Store, carved: &[usize]| (0..cells).map(|i| M::is_marked(store, carved, i, &mut NoOpLogger)).collect::<Vec<_>>();
        assert!(read(&store, &carved).iter().all(|&set| !set), "{}: a fresh mark is set", M::NAME);
        for i in (0..cells).filter(|&i| chosen(i)) { M::mark(&mut store, &mut carved, i, log); }
        assert_eq!(read(&store, &carved), (0..cells).map(chosen).collect::<Vec<_>>(), "{}", M::NAME);
        for i in (0..cells).filter(|&i| chosen(i)) { M::unmark(&mut store, &mut carved, i, log); }
        assert!(read(&store, &carved).iter().all(|&set| !set), "{}: a cleared mark is set", M::NAME);
        assert_eq!(carved, sorted, "{}: the carved region is not as it was", M::NAME);
        M::free::<usize, _>(&store, log);
    }

    #[test]
    fn every_filler_holds_a_mark_where_it_was_set() {
        for cells in [0, 1, 2, 63, 64, 65, 200] {
            marks_hold::<ByteMarks>(cells);
            marks_hold::<BitMarks>(cells);
            marks_hold::<InArrayMarks>(cells);
        }
    }

    #[test]
    fn the_bits_pack_into_words_and_the_pairs_allocate_nothing() {
        assert_eq!(BitMarks::create::<usize, _>(200, &mut NoOpLogger).len(), 200usize.div_ceil(WORD));
        assert_eq!(ByteMarks::create::<usize, _>(200, &mut NoOpLogger).len(), 200);
        assert_eq!((InArrayMarks::CARVED, InArrayMarks::ALLOCATED_BITS), (2, 0));
    }
}
