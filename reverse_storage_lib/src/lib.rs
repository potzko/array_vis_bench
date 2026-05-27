//! How the per-node *reverse* bits are stored.
//!
//! A weak heap of size `n` carries one boolean per node. Two ways to lay
//! them out in memory:
//!
//! - [`ByteStorage`]: one `u8` per node. Fast access (no shifting), but
//!   wastes 7 bits per node. The visualiser shows each cell as 0 or 1, so
//!   the scale is fixed at 1 and a flip toggles a cell between two
//!   states.
//! - [`BitStorage`]: 8 nodes packed per `u8`. Access costs a shift + mask;
//!   writes are a load-xor-store. Memory footprint drops 8×. The
//!   visualiser renders each *byte* (not each bit) so the scale is fixed
//!   at 255 and a single bit flip changes its byte's height by 2^bit_index
//!   — the column behaves like a binary counter as bits in the byte flip
//!   on and off.

use sort_logger::SortLogger;

pub trait ReverseStorage {
    /// Allocate storage backing `n` reverse bits, register it with the
    /// visualiser, and pin the scale.
    fn new<T: Ord + Copy, U: ?Sized + SortLogger<T>>(n: usize, logger: &mut U) -> Vec<u8>;

    /// Free the allocation in the visualiser.
    fn drop<T: Ord + Copy, U: ?Sized + SortLogger<T>>(state: Vec<u8>, logger: &mut U);

    /// Read bit `i`. Returns 0 or 1.
    fn get(state: &[u8], i: usize) -> u8;

    /// Toggle bit `i` and route the write through the logger so the
    /// visualiser observes the change.
    fn flip<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        state: &mut [u8],
        i: usize,
        logger: &mut U,
    );
}

pub struct ByteStorage;

impl ReverseStorage for ByteStorage {
    #[inline]
    fn new<T: Ord + Copy, U: ?Sized + SortLogger<T>>(n: usize, logger: &mut U) -> Vec<u8> {
        let reverse = vec![0u8; n];
        logger.log_aux_arr_u8(&reverse);
        reverse
    }

    #[inline]
    fn drop<T: Ord + Copy, U: ?Sized + SortLogger<T>>(state: Vec<u8>, logger: &mut U) {
        logger.free_aux_arr_u8(&state);
    }

    #[inline(always)]
    fn get(state: &[u8], i: usize) -> u8 {
        state[i]
    }

    #[inline(always)]
    fn flip<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        state: &mut [u8],
        i: usize,
        logger: &mut U,
    ) {
        logger.write_data_u8(state, i, state[i] ^ 1);
    }
}

pub struct BitStorage;

impl ReverseStorage for BitStorage {
    #[inline]
    fn new<T: Ord + Copy, U: ?Sized + SortLogger<T>>(n: usize, logger: &mut U) -> Vec<u8> {
        let reverse = vec![0u8; n.div_ceil(8)];
        logger.log_aux_arr_u8(&reverse);
        reverse
    }

    #[inline]
    fn drop<T: Ord + Copy, U: ?Sized + SortLogger<T>>(state: Vec<u8>, logger: &mut U) {
        logger.free_aux_arr_u8(&state);
    }

    #[inline(always)]
    fn get(state: &[u8], i: usize) -> u8 {
        (state[i >> 3] >> (i & 7)) & 1
    }

    #[inline(always)]
    fn flip<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        state: &mut [u8],
        i: usize,
        logger: &mut U,
    ) {
        let byte_idx = i >> 3;
        let new_val = state[byte_idx] ^ (1u8 << (i & 7));
        logger.write_data_u8(state, byte_idx, new_val);
    }
}
