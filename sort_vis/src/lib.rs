mod img_tmp;
mod sub_image;

use sort_logger::SortLog;

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
