mod img_tmp;
mod sub_image;

use sort_logger::SortLog;

pub trait Visualizer {
    fn render(&mut self, arr: &[usize], name: usize, actions: &[SortLog<usize>]);
}

pub use img_tmp::{render_gif, Encoding, Mp4Config, Mp4Visualizer, Pacing};
