use crate::arr_name;
use crate::sort_log::SortLog;

/// Generate the four-method int-typed aux-array family
/// (`$log` / `$create` / `$free` / `$write_data`) for one concrete integer
/// element type `$ty`. All variants share the same `SortLog` events
/// (`CreateAuxArr` / `FreeAuxArr` / `WriteDataU` with `data as usize`), so the
/// visualiser handles any integer width uniformly. Each expansion produces
/// non-generic methods, preserving trait dyn-compatibility.
macro_rules! int_aux_family {
    ($ty:ty, $log:ident, $create:ident, $free:ident, $write_data:ident) => {
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
        #[inline(always)]
        fn $free(&mut self, arr: &[$ty]) {
            self.log(SortLog::FreeAuxArr {
                name: arr_name!(arr),
            })
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
    #[inline(always)]
    fn free_aux_arr_t(&mut self, arr: &[T]) {
        self.log(SortLog::FreeAuxArr {
            name: arr_name!(arr),
        })
    }
    // Integer-typed aux array families. Adding a new width is one line:
    //     int_aux_family!(u16, log_aux_arr_u16, create_aux_arr_u16, ...);
    int_aux_family!(usize, log_aux_arr_u, create_aux_arr, free_aux_arr, write_data_u);
    int_aux_family!(u8, log_aux_arr_u8, create_aux_arr_u8, free_aux_arr_u8, write_data_u8);

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
