use std::marker::PhantomData;

#[warn(dead_code)]
#[derive(Debug, Clone)]
pub enum SortLog<T: Copy> {
    Cmp {
        name: usize,
        ind_a: usize,
        ind_b: usize,
        result: bool,
    },
    CmpSingle {
        name: usize,
        ind_a: usize,
        result: bool,
    },
    Swap {
        name: usize,
        ind_a: usize,
        ind_b: usize,
    },
    Copy {
        name: usize,
        ind_a: usize,
        ind_b: usize,
    },
    Mark(String),
    CreateAuxArr {
        name: usize,
        length: usize,
    },
    CmpAcrossArrs {
        name_a: usize,
        ind_a: usize,
        name_b: usize,
        ind_b: usize,
        result: bool,
    },
    SwapAcrossArrs {
        name_a: usize,
        ind_a: usize,
        name_b: usize,
        ind_b: usize,
    },
    CopyAcrossArrs {
        name_a: usize,
        ind_a: usize,
        name_b: usize,
        ind_b: usize,
    },
    CmpOuterData {
        name: usize,
        ind: usize,
        data: T,
        result: bool,
    },
    WriteOutOfArr {
        name: usize,
        ind: usize,
        data: T,
    },
    Write {
        name: usize,
        ind_a: usize,
        ind_b: usize,
    },
}

pub trait SortLogger<T: Copy> {
    fn log(&mut self, _: SortLog<T>) {}
}

impl<T: Copy> SortLogger<T> for () {
    fn log(&mut self, _: SortLog<T>) {}
}

#[derive(Debug)]
pub struct VisualizerLogger<T: Copy> {
    pub type_ghost: std::marker::PhantomData<T>,
    pub log: Vec<SortLog<T>>,
}

impl<T: Copy> SortLogger<T> for VisualizerLogger<T> {
    fn log(&mut self, data: SortLog<T>) {
        self.log.push(data);
    }
}
