#[warn(dead_code)]
#[derive(Debug)]
pub enum SortLog {
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
        data_type_size: usize,
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
}

pub trait SortLogger {
    fn log(&mut self, _: SortLog) {}
}

impl SortLogger for () {
    fn log(&mut self, _: SortLog) {}
}

#[derive(Debug)]
pub struct VisualizerLogger {
    pub log: Vec<SortLog>,
}

impl SortLogger for VisualizerLogger {
    fn log(&mut self, data: SortLog) {
        self.log.push(data);
    }
}
