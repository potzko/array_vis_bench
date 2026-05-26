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
    /// Optional hint: declare the maximum value the array will ever hold,
    /// so the visualiser can fix its bar-height scale before any WriteData
    /// events arrive. Without this, `max` starts at 0 and the visualiser
    /// has to invalidate every column each time a write exceeds the
    /// current `max` — O(n²) over an ascending init sequence. With it
    /// the scale is correct from the first write, so each WriteData stays
    /// O(1).
    ///
    /// Not a hard cap: a later WriteData with `data > max` still grows
    /// the scale (and triggers the full-redraw). The hint is advisory.
    /// The NoOpLogger ignores it; only the visualiser cares.
    SetScale {
        name: usize,
        max: T,
    },
    /// `SetScale` for integer aux arrays (u8 / usize / future widths),
    /// matching the existing `WriteDataU` split. `max` is cast to `usize`
    /// at the boundary so every integer width shares one event.
    SetScaleU {
        name: usize,
        max: usize,
    },
}
