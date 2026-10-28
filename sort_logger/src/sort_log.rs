//! [`SortLog`] event variants and the [`arr_name!`] helper that
//! identifies an array by its base pointer for cross-event correlation.

/// Identify an array by its base pointer, cast to `usize`.
///
/// Used as the `name` field on every [`SortLog`] variant so the
/// visualiser can correlate events that touch the same logical array
/// (the main slice, an auxiliary scratch buffer, etc.) without
/// requiring algorithms to thread a stable id.
#[macro_export]
macro_rules! arr_name {
    ($arr: expr) => {
        $arr.as_ptr() as usize
    };
}

/// What a held region of an array is for, so a renderer can tell the two apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionKind {
    /// The merge buffer: swap space whose values are real elements.
    Buffer,
    /// The key region: distinct elements that tag blocks while they roll.
    Keys,
}

/// One observable operation in a sort run.
///
/// Each variant carries enough state for a downstream consumer (the
/// visualiser, a stats collector, a replay engine) to reconstruct the
/// operation without re-running the algorithm. The `name` field is the
/// array's identity (see [`arr_name!`]); index fields are interpreted
/// against that array.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortLog<T: Copy + PartialEq + Eq> {
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
    /// `start..start + len` of the array `name` is the buffer or the key region of
    /// the sort, not data, while it is held. The same event with `held` false
    /// releases it.
    Region {
        name: usize,
        start: usize,
        len: usize,
        kind: RegionKind,
        held: bool,
    },
}
