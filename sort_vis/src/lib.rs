//! Visualiser backend for the `array_vis_bench` workspace.
//!
//! Consumes a `&[SortLog<usize>]` event stream (produced by
//! `sort_logger::VisualizerLogger`) and renders it into an animated
//! frame sequence. The shipped renderer ([`Mp4Visualizer`]) writes an
//! MP4 file via the [`Mp4Config`] knobs ([`Encoding`], [`Pacing`],
//! [`COMMON_FRAMERATES`], [`COMMON_RESOLUTIONS`]).
//!
//! The [`Visualizer`] trait is intentionally tiny — one method,
//! `render(events)` — so an alternate backend (e.g. a GIF renderer or
//! a WASM canvas) can be slotted in without touching the producer side.

mod img_tmp;
mod sub_image;

use sort_logger::SortLog;

/// Render a sort's event log into some visual output.
///
/// Implementors decide the format (MP4, GIF, live canvas, …) and the
/// frame-pacing strategy. The log alone must fully describe what to
/// display — there is no back-channel for "initial state" or a
/// privileged "main" array.
pub trait Visualizer {
    /// Render the given event stream end-to-end. The log alone must fully
    /// describe what to display — no back-channel for "initial state" or
    /// privileged "main" array. The first `CreateAuxArr*` event in the
    /// log creates the first array; subsequent events populate / mutate
    /// it; tile layout falls out of which arrays are currently live.
    fn render(&mut self, actions: &[SortLog<usize>]);
}

pub use img_tmp::{
    Encoding, Mp4Config, Mp4Visualizer, Pacing, COMMON_FRAMERATES, COMMON_RESOLUTIONS,
};
