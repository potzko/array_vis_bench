#[macro_export]
macro_rules! arr_name {
    ($arr: expr) => {
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
    CmpDataU {
        name: usize,
        ind: usize,
        data: usize,
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
    WriteDataU {
        name: usize,
        ind: usize,
        data: usize,
    },
}
