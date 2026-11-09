//! The [`SortLogger`] trait — every sort algorithm in the workspace
//! calls into one of its methods to record an observable operation.
//!
//! Method shape is non-generic per `T` (parameterized only by the
//! trait's own `T`) so the trait stays dyn-compatible, which lets the
//! visualiser drive sort code through `&mut dyn SortLogger<T>` while
//! the bench path passes a monomorphic `&mut NoOpLogger`.
//!
//! Integer aux-array families (`*_aux_arr*` / `*_data_*`) are generated
//! by the `int_aux_family!` macro from a single declaration per type
//! width, sharing the same `SortLog` events so the visualiser handles
//! any width uniformly.

use crate::arr_name;
use crate::sort_log::{RegionKind, SortLog};

/// `buf` cut into the sections a `create_aux_arr_split*` call laid it out in,
/// one slice per section length.
pub fn split_sections<T, const K: usize>(mut buf: &mut [T], sections: [usize; K]) -> [&mut [T]; K] {
    sections.map(|len| {
        let (section, rest) = std::mem::take(&mut buf).split_at_mut(len);
        buf = rest;
        section
    })
}

// The start and length of each section of a buffer laid out as `sections`,
// leaving out an empty one: it would be named by where it starts, which is
// the next section's name too.
fn nonempty(sections: &[usize]) -> impl Iterator<Item = (usize, usize)> + '_ {
    sections
        .iter()
        .scan(0, |start, &len| { *start += len; Some((*start - len, len)) })
        .filter(|&(_, len)| len > 0)
}

// Announces each section of `buf`, laid out as `sections`, through `announce`,
// and puts every one after the first beside the first, so they share a row.
fn announce_in_a_row<A, T: Copy + Ord, L: SortLogger<T> + ?Sized>(
    logger: &mut L,
    buf: &[A],
    sections: &[usize],
    announce: impl Fn(&mut L, &[A]),
) {
    let mut first = None;
    for (start, len) in nonempty(sections) {
        announce(logger, &buf[start..start + len]);
        let name = arr_name!(buf[start..]);
        match first {
            Some(of) => logger.draw_beside(name, of),
            None => first = Some(name),
        }
    }
}

/// Generate the six-method int-typed aux-array family
/// (`$log` / `$create` / `$create_split` / `$free` / `$free_split` /
/// `$write_data`) for one concrete integer element type `$ty`. All variants
/// share the same `SortLog` events (`CreateAuxArr` / `FreeAuxArr` /
/// `WriteDataU` with `data as usize`), so the visualiser handles any integer
/// width uniformly. Each expansion produces non-generic methods, preserving
/// trait dyn-compatibility.
macro_rules! int_aux_family {
    ($ty:ty, $log:ident, $create:ident, $create_split:ident, $free:ident, $free_split:ident, $write_data:ident) => {
        #[inline(always)]
        fn $log(&mut self, arr: &[$ty]) {
            self.log(SortLog::CreateAuxArr {
                name: arr_name!(arr),
                length: arr.len(),
            })
        }
        #[inline(always)]
        fn $create(&mut self, len: usize) -> Vec<$ty> {
            let ret = vec![0 as $ty; len];
            self.$log(&ret);
            ret
        }
        /// One allocation of the `sections` laid end to end, announced as an
        /// array per non empty section in one row, so the visualiser draws them
        /// inline, each to its own scale. Cut it with [`split_sections`].
        #[inline(always)]
        fn $create_split(&mut self, sections: &[usize]) -> Vec<$ty> {
            let ret = vec![0 as $ty; sections.iter().sum()];
            announce_in_a_row(self, &ret, sections, |logger, section| logger.$log(section));
            ret
        }
        #[inline(always)]
        fn $free(&mut self, arr: &[$ty]) {
            self.log(SortLog::FreeAuxArr {
                name: arr_name!(arr),
            })
        }
        #[inline(always)]
        fn $free_split(&mut self, arr: &[$ty], sections: &[usize]) {
            for (start, _) in nonempty(sections) { self.$free(&arr[start..]); }
        }
        #[inline(always)]
        fn $write_data(&mut self, arr: &mut [$ty], ind: usize, data: $ty) {
            self.log(SortLog::WriteDataU {
                name: arr_name!(arr),
                ind,
                data: data as usize,
            });
            arr[ind] = data;
        }
    };
}

/// Instrumentation trait for sort algorithms.
///
/// All methods are dyn-compatible — the trait can be used as `dyn SortLogger<T>`
/// without restriction.
pub trait SortLogger<T: Copy + Ord> {
    fn log(&mut self, _: SortLog<T>) {}

    /* Misc */
    #[inline(always)]
    fn mark(&mut self, mssg: String) {
        self.log(SortLog::Mark(mssg))
    }
    #[inline(always)]
    fn mark_mssg(&mut self, mssg: &str) {
        self.log(SortLog::Mark(mssg.to_string()))
    }
    /// `arr[start..start + len]` is now the buffer or the key region of the sort,
    /// not data, until [`release_region`](SortLogger::release_region).
    #[inline(always)]
    fn hold_region(&mut self, arr: &[T], start: usize, len: usize, kind: RegionKind) {
        self.log(SortLog::Region { name: arr_name!(arr), start, len, kind, held: true })
    }
    #[inline(always)]
    fn release_region(&mut self, arr: &[T], start: usize, len: usize, kind: RegionKind) {
        self.log(SortLog::Region { name: arr_name!(arr), start, len, kind, held: false })
    }
    #[inline(always)]
    fn log_aux_arr_t(&mut self, arr: &[T]) {
        self.log(SortLog::CreateAuxArrT {
            name: arr_name!(arr),
            length: arr.len(),
        })
    }
    #[inline(always)]
    fn copy_aux_arr_t(&mut self, arr: &[T]) -> Vec<T> {
        let mut ret = Vec::<T>::with_capacity(arr.len());
        unsafe { ret.set_len(arr.len()) }
        self.log_aux_arr_t(&ret);
        for i in 0..arr.len() {
            self.write_accross(arr, i, &mut ret, i)
        }
        ret
    }
    #[inline(always)]
    fn create_aux_arr_t(&mut self, len: usize) -> Vec<T> {
        let mut ret = Vec::<T>::with_capacity(len);
        unsafe { ret.set_len(len) }
        self.log_aux_arr_t(&ret);
        ret
    }
    #[inline(always)]
    fn copy_aux_arr(&mut self, arr: &[usize]) -> Vec<usize> {
        let mut ret = Vec::<usize>::with_capacity(arr.len());
        unsafe { ret.set_len(arr.len()) }
        self.log_aux_arr_u(&ret);
        for i in 0..arr.len() {
            self.write_accross_u(arr, i, &mut ret, i)
        }
        ret
    }
    /// One allocation of the `sections` laid end to end, announced as an array
    /// per non empty section in one row, so the visualiser draws them inline,
    /// each to its own scale. Cut it with [`split_sections`].
    #[inline(always)]
    fn create_aux_arr_split_t(&mut self, sections: &[usize]) -> Vec<T> {
        let len = sections.iter().sum();
        let mut ret = Vec::<T>::with_capacity(len);
        unsafe { ret.set_len(len) }
        announce_in_a_row(self, &ret, sections, |logger, section| logger.log_aux_arr_t(section));
        ret
    }
    /// Draw the row holding the array named `name` in the row of the array named
    /// `of`, names being [`arr_name!`] addresses, so arrays of two element
    /// types can share a row.
    #[inline(always)]
    fn draw_beside(&mut self, name: usize, of: usize) {
        self.log(SortLog::Beside { name, of })
    }
    #[inline(always)]
    fn free_aux_arr_t(&mut self, arr: &[T]) {
        self.log(SortLog::FreeAuxArr {
            name: arr_name!(arr),
        })
    }
    #[inline(always)]
    fn free_aux_arr_split_t(&mut self, arr: &[T], sections: &[usize]) {
        for (start, _) in nonempty(sections) { self.free_aux_arr_t(&arr[start..]); }
    }
    // Integer-typed aux array families. Adding a new width is one line:
    //     int_aux_family!(u16, log_aux_arr_u16, create_aux_arr_u16, ...);
    int_aux_family!(usize, log_aux_arr_u, create_aux_arr, create_aux_arr_split, free_aux_arr, free_aux_arr_split, write_data_u);
    int_aux_family!(u8, log_aux_arr_u8, create_aux_arr_u8, create_aux_arr_split_u8, free_aux_arr_u8, free_aux_arr_split_u8, write_data_u8);

    /*----------------
        Cmps
    --------------- */

    #[inline(always)]
    fn cmp_eq(&mut self, arr: &[T], ind_a: usize, ind_b: usize) -> bool {
        let result = arr[ind_a] == arr[ind_b];
        self.log(SortLog::CmpInArr {
            name: arr_name!(arr),
            ind_a,
            ind_b,
            result,
        });
        result
    }
    #[inline(always)]
    fn cmp_neq(&mut self, arr: &[T], ind_a: usize, ind_b: usize) -> bool {
        let result = arr[ind_a] != arr[ind_b];
        self.log(SortLog::CmpInArr {
            name: arr_name!(arr),
            ind_a,
            ind_b,
            result,
        });
        result
    }
    #[inline(always)]
    fn cmp_lt(&mut self, arr: &[T], ind_a: usize, ind_b: usize) -> bool {
        let result = arr[ind_a] < arr[ind_b];
        self.log(SortLog::CmpInArr {
            name: arr_name!(arr),
            ind_a,
            ind_b,
            result,
        });
        result
    }
    #[inline(always)]
    fn cmp_le(&mut self, arr: &[T], ind_a: usize, ind_b: usize) -> bool {
        let result = arr[ind_a] <= arr[ind_b];
        self.log(SortLog::CmpInArr {
            name: arr_name!(arr),
            ind_a,
            ind_b,
            result,
        });
        result
    }
    #[inline(always)]
    fn cmp_gt(&mut self, arr: &[T], ind_a: usize, ind_b: usize) -> bool {
        let result = arr[ind_a] > arr[ind_b];
        self.log(SortLog::CmpInArr {
            name: arr_name!(arr),
            ind_a,
            ind_b,
            result,
        });
        result
    }
    #[inline(always)]
    fn cmp_ge(&mut self, arr: &[T], ind_a: usize, ind_b: usize) -> bool {
        let result = arr[ind_a] >= arr[ind_b];
        self.log(SortLog::CmpInArr {
            name: arr_name!(arr),
            ind_a,
            ind_b,
            result,
        });
        result
    }

    // in-arr to outside data cmp
    #[inline(always)]
    fn cmp_eq_data(&mut self, arr: &[T], ind: usize, data: T) -> bool {
        let result = arr[ind] == data;
        self.log(SortLog::CmpData {
            name: arr_name!(arr),
            ind,
            data,
            result,
        });
        result
    }
    #[inline(always)]
    fn cmp_neq_data(&mut self, arr: &[T], ind: usize, data: T) -> bool {
        let result = arr[ind] != data;
        self.log(SortLog::CmpData {
            name: arr_name!(arr),
            ind,
            data,
            result,
        });
        result
    }
    #[inline(always)]
    fn cmp_lt_data(&mut self, arr: &[T], ind: usize, data: T) -> bool {
        let result = arr[ind] < data;
        self.log(SortLog::CmpData {
            name: arr_name!(arr),
            ind,
            data,
            result,
        });
        result
    }
    #[inline(always)]
    fn cmp_le_data(&mut self, arr: &[T], ind: usize, data: T) -> bool {
        let result = arr[ind] <= data;
        self.log(SortLog::CmpData {
            name: arr_name!(arr),
            ind,
            data,
            result,
        });
        result
    }
    #[inline(always)]
    fn cmp_gt_data(&mut self, arr: &[T], ind: usize, data: T) -> bool {
        let result = arr[ind] > data;
        self.log(SortLog::CmpData {
            name: arr_name!(arr),
            ind,
            data,
            result,
        });
        result
    }
    #[inline(always)]
    fn cmp_gt_data_u(&mut self, arr: &[usize], ind: usize, data: usize) -> bool {
        let result = arr[ind] > data;
        self.log(SortLog::CmpDataU {
            name: arr_name!(arr),
            ind,
            data,
            result,
        });
        result
    }
    #[inline(always)]
    fn cmp_ge_data(&mut self, arr: &[T], ind: usize, data: T) -> bool {
        let result = arr[ind] >= data;
        self.log(SortLog::CmpData {
            name: arr_name!(arr),
            ind,
            data,
            result,
        });
        result
    }

    // arr_a to arr_b cmp
    #[inline(always)]
    fn cmp_lt_accross(&mut self, arr_a: &[T], ind_a: usize, arr_b: &[T], ind_b: usize) -> bool {
        let result = arr_a[ind_a] < arr_b[ind_b];
        self.log(SortLog::CmpAcrossArrs {
            name_a: arr_name!(arr_a),
            ind_a,
            name_b: arr_name!(arr_b),
            ind_b,
            result,
        });
        result
    }
    #[inline(always)]
    fn cmp_le_accross(&mut self, arr_a: &[T], ind_a: usize, arr_b: &[T], ind_b: usize) -> bool {
        let result = arr_a[ind_a] <= arr_b[ind_b];
        self.log(SortLog::CmpAcrossArrs {
            name_a: arr_name!(arr_a),
            ind_a,
            name_b: arr_name!(arr_b),
            ind_b,
            result,
        });
        result
    }
    #[inline(always)]
    fn cmp_gt_accross(&mut self, arr_a: &[T], ind_a: usize, arr_b: &[T], ind_b: usize) -> bool {
        self.cmp_lt_accross(arr_b, ind_b, arr_a, ind_a)
    }
    #[inline(always)]
    fn cmp_ge_accross(&mut self, arr_a: &[T], ind_a: usize, arr_b: &[T], ind_b: usize) -> bool {
        self.cmp_le_accross(arr_b, ind_b, arr_a, ind_a)
    }

    /*----------------
        Writes
    --------------- */
    #[inline(always)]
    fn write(&mut self, arr: &mut [T], ind_a: usize, ind_b: usize) {
        self.log(SortLog::WriteInArr {
            name: arr_name!(arr),
            ind_a,
            ind_b,
        });
        arr[ind_a] = arr[ind_b]
    }
    #[inline(always)]
    fn write_u(&mut self, arr: &mut [usize], ind_a: usize, ind_b: usize) {
        self.log(SortLog::WriteInArr {
            name: arr_name!(arr),
            ind_a,
            ind_b,
        });
        arr[ind_a] = arr[ind_b]
    }
    #[inline(always)]
    fn write_data(&mut self, arr: &mut [T], ind: usize, data: T) {
        self.log(SortLog::WriteData {
            name: arr_name!(arr),
            ind,
            data,
        });
        arr[ind] = data;
    }
    #[inline(always)]
    fn write_accross(&mut self, arr_a: &[T], ind_a: usize, arr_b: &mut [T], ind_b: usize) {
        self.write_data(arr_b, ind_b, arr_a[ind_a]);
    }
    #[inline(always)]
    fn write_accross_u(
        &mut self,
        arr_a: &[usize],
        ind_a: usize,
        arr_b: &mut [usize],
        ind_b: usize,
    ) {
        self.write_data_u(arr_b, ind_b, arr_a[ind_a]);
    }

    /*----------------
        Swaps — all dyn-compatible
    --------------- */
    #[inline(always)]
    fn swap(&mut self, arr: &mut [T], ind_a: usize, ind_b: usize) {
        self.log(SortLog::Swap {
            name: arr_name!(arr),
            ind_a,
            ind_b,
        });
        arr.swap(ind_a, ind_b);
    }

    /// Conditionally swap arr[ind_a] and arr[ind_b] if arr[ind_a] < arr[ind_b].
    /// Returns true if a swap occurred.
    #[inline(always)]
    fn cond_swap_lt(&mut self, arr: &mut [T], ind_a: usize, ind_b: usize) -> bool {
        let result = arr[ind_a] < arr[ind_b];
        self.log(SortLog::CmpInArr {
            name: arr_name!(arr),
            ind_a,
            ind_b,
            result,
        });
        if result {
            self.log(SortLog::Swap {
                name: arr_name!(arr),
                ind_a,
                ind_b,
            });
            arr.swap(ind_a, ind_b);
        }
        result
    }

    /// Conditionally swap arr[ind_a] and arr[ind_b] if arr[ind_a] <= arr[ind_b].
    #[inline(always)]
    fn cond_swap_le(&mut self, arr: &mut [T], ind_a: usize, ind_b: usize) -> bool {
        let result = arr[ind_a] <= arr[ind_b];
        self.log(SortLog::CmpInArr {
            name: arr_name!(arr),
            ind_a,
            ind_b,
            result,
        });
        if result {
            self.log(SortLog::Swap {
                name: arr_name!(arr),
                ind_a,
                ind_b,
            });
            arr.swap(ind_a, ind_b);
        }
        result
    }

    /// Conditionally swap arr[ind_a] and arr[ind_b] if arr[ind_a] >= arr[ind_b].
    #[inline(always)]
    fn cond_swap_ge(&mut self, arr: &mut [T], ind_a: usize, ind_b: usize) -> bool {
        let result = arr[ind_a] >= arr[ind_b];
        self.log(SortLog::CmpInArr {
            name: arr_name!(arr),
            ind_a,
            ind_b,
            result,
        });
        if result {
            self.log(SortLog::Swap {
                name: arr_name!(arr),
                ind_a,
                ind_b,
            });
            arr.swap(ind_a, ind_b);
        }
        result
    }

    /// Conditionally swap arr[ind_a] and arr[ind_b] if arr[ind_a] > arr[ind_b].
    #[inline(always)]
    fn cond_swap_gt(&mut self, arr: &mut [T], ind_a: usize, ind_b: usize) -> bool {
        let result = arr[ind_a] > arr[ind_b];
        self.log(SortLog::CmpInArr {
            name: arr_name!(arr),
            ind_a,
            ind_b,
            result,
        });
        if result {
            self.log(SortLog::Swap {
                name: arr_name!(arr),
                ind_a,
                ind_b,
            });
            arr.swap(ind_a, ind_b);
        }
        result
    }

    /*----------------
        Compound operations — all dyn-compatible
    --------------- */

    /// Reverse a slice in-place.
    #[inline(always)]
    fn reverse(&mut self, arr: &mut [T]) {
        let n = arr.len();
        let mut i = 0;
        let mut ii = n.saturating_sub(1);
        while i < ii {
            self.swap(arr, i, ii);
            i += 1;
            ii -= 1;
        }
    }

    /// Swap `n` elements starting at `s1` with `n` elements starting at `s2`.
    #[inline(always)]
    fn block_swap(&mut self, arr: &mut [T], s1: usize, s2: usize, n: usize) {
        for i in 0..n {
            self.swap(arr, s1 + i, s2 + i);
        }
    }

    /// Shift `arr[to..from]` right by one slot and write `data` at `arr[to]`.
    #[inline(always)]
    fn shift_insert(&mut self, arr: &mut [T], from: usize, to: usize, data: T) {
        let mut j = from;
        while j > to {
            let v = arr[j - 1];
            self.write_data(arr, j, v);
            j -= 1;
        }
        self.write_data(arr, to, data);
    }

    /// Copy `len` elements from `src[src_start..]` into `dst[dst_start..]`.
    #[inline(always)]
    fn copy_range(&mut self, src: &[T], src_start: usize, dst: &mut [T], dst_start: usize, len: usize) {
        for i in 0..len {
            self.write_accross(src, src_start + i, dst, dst_start + i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VisualizerLogger;
    use std::marker::PhantomData;

    #[test]
    fn a_region_is_held_and_released_by_one_event_each() {
        let arr = vec![0usize; 10];
        let name = arr.as_ptr() as usize;
        let mut log = VisualizerLogger::<usize> { type_ghost: PhantomData, log: Vec::new() };
        log.hold_region(&arr, 2, 3, RegionKind::Keys);
        log.release_region(&arr, 2, 3, RegionKind::Keys);
        assert_eq!(
            log.log,
            [
                SortLog::Region { name, start: 2, len: 3, kind: RegionKind::Keys, held: true },
                SortLog::Region { name, start: 2, len: 3, kind: RegionKind::Keys, held: false },
            ]
        );
    }

    #[test]
    fn a_buffer_in_sections_is_an_array_per_section_in_one_row() {
        let mut log = VisualizerLogger::<usize> { type_ghost: PhantomData, log: Vec::new() };
        let words = log.create_aux_arr_split(&[3, 0, 2]);
        let elems = log.create_aux_arr_split_t(&[4]);
        let (base, tail, whole) = (words.as_ptr() as usize, words[3..].as_ptr() as usize, elems.as_ptr() as usize);
        log.draw_beside(whole, base);
        log.free_aux_arr_split(&words, &[3, 0, 2]);
        log.free_aux_arr_split_t(&elems, &[4]);
        // the empty section is left out, its name would be the next one's
        assert_eq!(words, [0; 5]);
        assert_eq!(
            log.log,
            [
                SortLog::CreateAuxArr { name: base, length: 3 },
                SortLog::CreateAuxArr { name: tail, length: 2 },
                SortLog::Beside { name: tail, of: base },
                SortLog::CreateAuxArrT { name: whole, length: 4 },
                SortLog::Beside { name: whole, of: base },
                SortLog::FreeAuxArr { name: base },
                SortLog::FreeAuxArr { name: tail },
                SortLog::FreeAuxArr { name: whole },
            ]
        );
    }

    #[test]
    fn a_buffer_splits_into_the_sections_it_was_created_with() {
        let mut buf = [1, 2, 3, 4, 5];
        let [head, empty, tail] = crate::split_sections(&mut buf, [3, 0, 2]);
        assert_eq!((&*head, &*empty, &*tail), (&[1, 2, 3][..], &[][..], &[4, 5][..]));
    }

    #[test]
    fn a_region_costs_nothing_to_a_logger_that_drops_events() {
        let arr = vec![0usize; 4];
        let mut quiet = crate::NoOpLogger;
        quiet.hold_region(&arr, 0, 4, RegionKind::Buffer);
        quiet.release_region(&arr, 0, 4, RegionKind::Buffer);
    }
}
