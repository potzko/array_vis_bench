#[macro_export]
macro_rules! arr_name {
    (  $arr: expr ) => {
        $arr.as_ptr() as usize
    };
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum SortLog<T: Copy> {
    Swap {
        name: usize,
        ind_a: usize,
        ind_b: usize,
    },
    Mark(String),
    CreateAuxArrT {
        name: usize,
        length: usize,
    },
    CreateAuxArr {
        name: usize,
        length: usize,
    },
    FreeAuxArr {
        name: usize,
    },
    CmpInArr {
        name: usize,
        ind_a: usize,
        ind_b: usize,
        result: bool,
    },
    CmpData {
        name: usize,
        ind: usize,
        data: T,
        result: bool,
    },
    CmpAcrossArrs {
        name_a: usize,
        ind_a: usize,
        name_b: usize,
        ind_b: usize,
        result: bool,
    },
    WriteInArr {
        name: usize,
        ind_a: usize,
        ind_b: usize,
    },
    WriteData {
        name: usize,
        ind: usize,
        data: T,
    },
    WritAcrossArrs {
        name_a: usize,
        ind_a: usize,
        name_b: usize,
        ind_b: usize,
    },
}

pub trait SortLogger<T: Copy + Ord> {
    fn log(&mut self, _: SortLog<T>) {}

    /* Misc */
    #[inline]
    fn mark(&mut self, mssg: String) {
        self.log(SortLog::Mark(mssg))
    }
    #[inline]
    fn mark_mssg(&mut self, mssg: &str) {
        self.log(SortLog::Mark(mssg.to_string()))
    }
    #[inline]
    fn create_aux_arr_t(&mut self, arr: &[T]) {
        self.log(SortLog::CreateAuxArrT {
            name: arr_name!(arr),
            length: arr.len(),
        })
    }
    #[inline]
    fn create_aux_arr(&mut self, arr: &[usize]) {
        self.log(SortLog::CreateAuxArrT {
            name: arr_name!(arr),
            length: arr.len(),
        })
    }
    #[inline]
    fn free_aux_arr_t(&mut self, arr: &[T]) {
        self.log(SortLog::FreeAuxArr {
            name: arr_name!(arr),
        })
    }
    #[inline]
    fn free_aux_arr(&mut self, arr: &[usize]) {
        self.log(SortLog::FreeAuxArr {
            name: arr_name!(arr),
        })
    }

    /*----------------
        Cmps
        lt -> less then
        le -> less then or equal
        gt -> greater then
        ge -> greater then or equal
    --------------- */

    //in arr cmp
    #[inline]
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
    #[inline]
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
    #[inline]
    fn cmp_gt(&mut self, arr: &[T], ind_a: usize, ind_b: usize) -> bool {
        self.cmp_lt(arr, ind_b, ind_a)
    }
    #[inline]
    fn cmp_ge(&mut self, arr: &[T], ind_a: usize, ind_b: usize) -> bool {
        self.cmp_le(arr, ind_b, ind_a)
    }

    //in arr to outside data cmp
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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

    //arr_a to arr_b cmp
    #[inline]
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
    #[inline]
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
    #[inline]
    fn cmp_gt_accross(&mut self, arr_a: &[T], ind_a: usize, arr_b: &[T], ind_b: usize) -> bool {
        self.cmp_lt_accross(arr_b, ind_b, arr_a, ind_a)
    }
    #[inline]
    fn cmp_ge_accross(&mut self, arr_a: &[T], ind_a: usize, arr_b: &[T], ind_b: usize) -> bool {
        self.cmp_le_accross(arr_b, ind_b, arr_a, ind_a)
    }

    /*----------------
        Writes
    --------------- */
    #[inline]
    fn write(&mut self, arr: &mut [T], ind_a: usize, ind_b: usize) {
        self.log(SortLog::WriteInArr {
            name: arr_name!(arr),
            ind_a,
            ind_b,
        });
        arr[ind_a] = arr[ind_b]
    }
    #[inline]
    fn write_data(&mut self, arr: &mut [T], ind: usize, data: T) {
        self.log(SortLog::WriteData {
            name: arr_name!(arr),
            ind,
            data,
        });
        arr[ind] = data;
    }
    #[inline]
    fn write_accross(&mut self, arr_a: &mut [T], ind_a: usize, arr_b: &mut [T], ind_b: usize) {
        self.log(SortLog::WritAcrossArrs {
            name_a: arr_name!(arr_a),
            ind_a,
            name_b: arr_name!(arr_b),
            ind_b,
        })
    }

    /*----------------
        Swaps
    --------------- */
    #[inline]
    fn swap(&mut self, arr: &mut [T], ind_a: usize, ind_b: usize) {
        self.log(SortLog::Swap {
            name: arr_name!(arr),
            ind_a,
            ind_b,
        });
        arr.swap(ind_a, ind_b);
    }
    #[inline]
    fn cond_swap_lt(&mut self, arr: &mut [T], ind_a: usize, ind_b: usize) -> bool {
        let ret = self.cmp_lt(arr, ind_a, ind_b);
        if ret {
            self.swap(arr, ind_a, ind_b)
        }
        ret
    }
    #[inline]
    fn cond_swap_le(&mut self, arr: &mut [T], ind_a: usize, ind_b: usize) -> bool {
        let ret = self.cmp_le(arr, ind_a, ind_b);
        if ret {
            self.swap(arr, ind_a, ind_b)
        }
        ret
    }
    #[inline]
    fn cond_swap_ge(&mut self, arr: &mut [T], ind_a: usize, ind_b: usize) -> bool {
        let ret = self.cmp_ge(arr, ind_a, ind_b);
        if ret {
            self.swap(arr, ind_a, ind_b)
        }
        ret
    }
    #[inline]
    fn cond_swap_gt(&mut self, arr: &mut [T], ind_a: usize, ind_b: usize) -> bool {
        let ret = self.cmp_gt(arr, ind_a, ind_b);
        if ret {
            self.swap(arr, ind_a, ind_b)
        }
        ret
    }
}

impl<T: Copy + Ord> SortLogger<T> for () {
    #[inline]
    fn log(&mut self, _: SortLog<T>) {}
}

#[derive(Debug)]
pub struct VisualizerLogger<T: Copy + Ord> {
    pub type_ghost: std::marker::PhantomData<T>,
    pub log: Vec<SortLog<T>>,
}

impl<T: Copy + Ord> SortLogger<T> for VisualizerLogger<T> {
    #[inline]
    fn log(&mut self, data: SortLog<T>) {
        self.log.push(data);
    }
}
