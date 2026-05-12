#![allow(dead_code)]

use crate::sub_image::{Framebuffer, SubImg};
use crate::Visualizer;
use sort_logger::SortLog;
use std::collections::BTreeMap;
use std::hash::Hash;
use std::io::Write;
use std::mem::size_of;
use std::process::{Child, Command, Stdio};

/// Lossless — perfect quality, large file, moderate encode speed.
/// Lossy     — good quality (CRF 23), small file, moderate encode speed.
/// Fast      — lower quality (CRF 28), small file, fastest encode.
pub enum Encoding {
    Lossless,
    Lossy,
    Fast,
}

/// How `SortLog` actions are paced into video frames.
pub enum Pacing {
    /// Fixed: this many actions per rendered frame. Larger value → faster
    /// visualisation (more action per frame, fewer frames overall).
    ActionsPerFrame(usize),
    /// Target the full animation to take this long; `actions_per_frame`
    /// is computed at render time from `framerate * seconds` and the
    /// total log length.
    DurationSeconds(f64),
}

pub struct Mp4Config {
    pub output_width: u32,
    pub output_height: u32,
    pub framerate: u32,
    pub pacing: Pacing,
    pub encoding: Encoding,
    pub output_path: String,
}

/// Common video resolutions: `(width, height, label)`.
pub const COMMON_RESOLUTIONS: &[(u32, u32, &str)] = &[
    (1280, 720,  "720p (HD)"),
    (1920, 1080, "1080p (Full HD / 2K)"),
    (2560, 1440, "1440p (QHD)"),
    (3840, 2160, "2160p (4K UHD)"),
];

/// Common frame rates.
pub const COMMON_FRAMERATES: &[u32] = &[30, 60, 120];

pub struct Mp4Visualizer {
    pub config: Mp4Config,
}

const WHITE: [u8; 3] = [0xff, 0xff, 0xff];
const BLACK: [u8; 3] = [0x00, 0x00, 0x00];
const GREEN: [u8; 3] = [0x00, 0xa0, 0x60];
const BLUE:  [u8; 3] = [0x00, 0x30, 0xff];

fn get_views(view: &SubImg, amount: u32) -> Vec<SubImg> {
    let height = view.height / amount;
    (0..amount)
        .map(|i| view.make_sub_img(0, i * height, view.width, height))
        .collect()
}

impl Mp4Visualizer {
    pub fn new(config: Mp4Config) -> Self {
        Self { config }
    }

    fn spawn_ffmpeg(&self) -> Child {
        let video_size = format!("{}x{}", self.config.output_width, self.config.output_height);
        let framerate = self.config.framerate.to_string();

        // Args shared by all modes: input spec
        let mut args: Vec<&str> = vec![
            "-loglevel", "error",
            "-y",
            "-f", "rawvideo",
            "-pixel_format", "rgb24",
            "-video_size", &video_size,
            "-framerate", &framerate,
            "-i", "pipe:0",
        ];

        // Mode-specific output args
        match self.config.encoding {
            Encoding::Lossless => args.extend([
                // libx264rgb stores raw RGB — no YUV conversion, no quality loss.
                // colormatrix=GBR signals to players not to apply a YUV matrix on playback.
                "-c:v", "libx264rgb",
                "-crf", "0",
                "-preset", "medium",
                "-x264-params", "colormatrix=GBR:colorprim=bt709:transfer=bt709",
            ]),
            Encoding::Lossy => args.extend([
                "-c:v", "libx264",
                "-pix_fmt", "yuv444p",  // 4:4:4 avoids chroma blur on sharp edges
                "-crf", "23",
                "-preset", "medium",
            ]),
            Encoding::Fast => args.extend([
                "-c:v", "libx264",
                "-pix_fmt", "yuv420p",
                "-crf", "28",
                "-preset", "ultrafast",
            ]),
        }

        args.push(&self.config.output_path);

        Command::new("ffmpeg")
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("failed to spawn ffmpeg — is it installed?")
    }
}

impl Visualizer for Mp4Visualizer {
    fn render(&mut self, arr: &[usize], name: usize, actions: &[SortLog<usize>]) {
        let actions_per_frame = match self.config.pacing {
            Pacing::ActionsPerFrame(n) => n.max(1),
            Pacing::DurationSeconds(s) => {
                let target_frames = (s * self.config.framerate as f64).round() as usize;
                (actions.len() / target_frames.max(1)).max(1)
            }
        };
        let arr = arr.to_vec();

        let mut inplace = true;
        for ev in actions {
            match ev {
                SortLog::CreateAuxArr { .. } | SortLog::CreateAuxArrT { .. } => {
                    inplace = false;
                    break;
                }
                _ => {}
            }
        }

        let out_w = self.config.output_width;
        let out_h = self.config.output_height;
        println!(
            "{} frames to generate ({} actions, {} actions/frame)",
            actions.len() / actions_per_frame + 3,
            actions.len(),
            actions_per_frame
        );

        let mut fb = Framebuffer::new(out_w, out_h);

        let view = SubImg {
            x: 0,
            y: if inplace { 0 } else { out_h / 2 },
            width: out_w,
            height: if inplace { out_h } else { out_h / 2 },
        };
        let aux_view = SubImg {
            x: 0,
            y: 0,
            width: out_w,
            height: out_h / 2,
        };
        let mut store = ArrStore::new(ArrActions::new(arr, view, name));
        // Paint the main array's initial state into the framebuffer.
        store.main_mut().force_full_redraw(&mut fb);

        let mut ffmpeg = self.spawn_ffmpeg();
        let stdin = ffmpeg.stdin.as_mut().expect("ffmpeg stdin not available");

        let mut i = 1;
        while i * actions_per_frame - actions_per_frame < actions.len() {
            let mut split_points: Vec<usize> = vec![i * actions_per_frame - actions_per_frame];
            #[allow(clippy::needless_range_loop)]
            for ii in i * actions_per_frame - actions_per_frame
                ..std::cmp::min(i * actions_per_frame, actions.len())
            {
                match actions[ii] {
                    SortLog::CreateAuxArr { .. } => split_points.push(ii),
                    SortLog::CreateAuxArrT { .. } => split_points.push(ii),
                    SortLog::FreeAuxArr { .. } => split_points.push(ii),
                    _ => {}
                }
            }
            // Process all events up to and including each create/free split,
            // applying the layout change in place, but *without* emitting a
            // frame at the split. Transient auxes (created and freed within
            // the same batch) get their layout side-effects applied but never
            // claim a dedicated frame — only the end-of-batch state is shown.
            for ii in 1..split_points.len() {
                store.update(&actions[split_points[ii - 1]..split_points[ii]], &mut fb);

                match actions[split_points[ii]] {
                    SortLog::CreateAuxArrT { name, length }
                    | SortLog::CreateAuxArr { name, length } => {
                        store.insert(ArrActions::new(
                            vec![0; length],
                            SubImg { x: 0, y: 0, width: 0, height: 0 },
                            name,
                        ));
                        redistribute_aux_views(&mut store, &aux_view, &mut fb);
                    }
                    SortLog::FreeAuxArr { name } => {
                        store.remove(name);
                        redistribute_aux_views(&mut store, &aux_view, &mut fb);
                    }
                    _ => {}
                }
            }

            store.update(
                &actions[*split_points.last().unwrap()
                    ..std::cmp::min(i * actions_per_frame, actions.len())],
                &mut fb,
            );
            stdin.write_all(&fb.data).unwrap();

            i += 1;
            if i % 100 == 0 {
                println!("{i} of {}", actions.len() / actions_per_frame + 3);
            }
        }

        // Close stdin so ffmpeg knows the stream is done
        drop(ffmpeg.stdin.take());
        let status = ffmpeg.wait().expect("failed to wait on ffmpeg");
        if !status.success() {
            eprintln!("ffmpeg exited with status: {status}");
        }
    }
}

/// Blank the aux region and repaint every surviving aux array across a fresh
/// equal-share split of `aux_view`. Used after both create and free events so
/// freed auxes don't leave residual pixels on screen and surviving auxes
/// always occupy the full aux region.
fn redistribute_aux_views(store: &mut ArrStore, aux_view: &SubImg, fb: &mut Framebuffer) {
    aux_view.rect(fb, 0, 0, aux_view.width, aux_view.height, BLACK);
    let aux_count = store.aux_count();
    if aux_count == 0 {
        return;
    }
    let views = get_views(aux_view, aux_count as u32);
    for (aux, view) in store.aux_iter_mut().zip(views.into_iter()) {
        aux.set_view(view);
        aux.force_full_redraw(fb);
    }
}

// ---------------------------------------------------------------------------
// ArrStore — address-keyed BTreeMap; O(log N) dispatch per action
// ---------------------------------------------------------------------------

/// Holds all tracked arrays keyed by their base pointer (address). Any event
/// address is resolved to the owning array via `range(..=addr).next_back()`
/// — the array with the largest base ≤ the event address — followed by a
/// bounds check against that array's length.
///
/// The main sort array is identified by its base address (`main_addr`), not
/// by position in the container: aux allocations may land at addresses below
/// the main, so we cannot assume the main lives at any particular slot.
struct ArrStore {
    arrs: BTreeMap<usize, ArrActions>,
    main_addr: usize,
}

impl ArrStore {
    fn new(main: ArrActions) -> Self {
        let main_addr = main.name;
        let mut arrs = BTreeMap::new();
        arrs.insert(main_addr, main);
        ArrStore { arrs, main_addr }
    }

    /// Find the array whose memory range contains `addr`. Returns a mutable
    /// reference to it and the element offset within it.
    fn lookup_mut(&mut self, addr: usize) -> Option<(&mut ArrActions, usize)> {
        let size_t = size_of::<usize>();
        let (&base, a) = self.arrs.range_mut(..=addr).next_back()?;
        if addr < base + a.arr.len() * size_t {
            Some((a, (addr - base) / size_t))
        } else {
            None
        }
    }

    fn main_mut(&mut self) -> &mut ArrActions {
        self.arrs.get_mut(&self.main_addr).expect("main array missing")
    }

    fn insert(&mut self, entry: ArrActions) {
        self.arrs.insert(entry.name, entry);
    }

    fn remove(&mut self, name: usize) {
        self.arrs.remove(&name);
    }

    /// Number of aux arrays (excludes the main).
    fn aux_count(&self) -> usize {
        self.arrs.len() - 1
    }

    /// Iterator over aux arrays only, in address order.
    fn aux_iter_mut(&mut self) -> impl Iterator<Item = &mut ArrActions> + '_ {
        let main_addr = self.main_addr;
        self.arrs
            .iter_mut()
            .filter_map(move |(addr, a)| if *addr == main_addr { None } else { Some(a) })
    }

    /// Apply all actions (mark dirty / colored on the right array), then have
    /// every array repaint just the columns whose visible state changed.
    fn update(&mut self, actions: &[SortLog<usize>], fb: &mut Framebuffer) {
        for action in actions {
            match action {
                SortLog::Swap { name, ind_a, ind_b } => {
                    if let Some((a, off)) = self.lookup_mut(*name) {
                        let ia = ind_a + off;
                        let ib = ind_b + off;
                        a.arr.swap(ia, ib);
                        a.mark_dirty(ia);
                        a.mark_dirty(ib);
                    }
                }
                SortLog::WriteData { name, ind, data } => {
                    if let Some((a, off)) = self.lookup_mut(*name) {
                        let idx = ind + off;
                        a.arr[idx] = *data;
                        a.mark_dirty(idx);
                        let v = *data as f64;
                        if v < a.min { a.min = v; }
                        if v > a.max { a.max = v; }
                    }
                }
                SortLog::WriteDataU { name, ind, data } => {
                    if let Some((a, off)) = self.lookup_mut(*name) {
                        let idx = ind + off;
                        a.arr[idx] = *data;
                        a.mark_dirty(idx);
                        let v = *data as f64;
                        if v < a.min { a.min = v; }
                        if v > a.max { a.max = v; }
                    }
                }
                SortLog::WriteInArr { name, ind_a, ind_b } => {
                    if let Some((a, off)) = self.lookup_mut(*name) {
                        let ia = ind_a + off;
                        let ib = ind_b + off;
                        a.arr[ia] = a.arr[ib];
                        a.mark_dirty(ia);
                        a.mark_dirty(ib);
                    }
                }
                SortLog::CmpInArr { name, ind_a, ind_b, result: _ } => {
                    if let Some((a, off)) = self.lookup_mut(*name) {
                        a.mark_color(ind_a + off);
                        a.mark_color(ind_b + off);
                    }
                }
                SortLog::CmpData { name, ind, data: _, result: _ } => {
                    if let Some((a, off)) = self.lookup_mut(*name) {
                        a.mark_color(ind + off);
                    }
                }
                SortLog::CmpDataU { name, ind, data: _, result: _ } => {
                    if let Some((a, off)) = self.lookup_mut(*name) {
                        a.mark_color(ind + off);
                    }
                }
                SortLog::CmpAcrossArrs { name_a, ind_a, name_b, ind_b, result: _ } => {
                    if let Some((a, off)) = self.lookup_mut(*name_a) {
                        a.mark_color(ind_a + off);
                    }
                    if let Some((a, off)) = self.lookup_mut(*name_b) {
                        a.mark_color(ind_b + off);
                    }
                }
                _ => {}
            }
        }
        for a in self.arrs.values_mut() {
            a.finalize_frame(fb);
        }
    }
}

// ---------------------------------------------------------------------------
// ArrActions — per-array state and rendering
// ---------------------------------------------------------------------------
//
// Rendering uses a per-pixel-column density approach so no array values are
// ever dropped:
//   * Each pixel column `x` owns a *range* of array indices `[a_x, b_x)`.
//     For `n <= W` the range has length 1 and is repeated across the
//     pixel-columns owned by that index. For `n > W` each pixel column owns
//     two-or-so indices and they all contribute to that column's render.
//   * Each row `y` of a column shows `(bars_in_col_reaching_y / k)` blended
//     between BLACK and WHITE, with GREEN tinting added for the share of
//     bars reaching `y` that are flagged as currently being compared.
//   * Counts per row are computed in `O(k + H)` per column via a "diff"
//     array: each bar contributes `+1` at its top row, then the running
//     prefix sum as we walk down gives the cover count at every row.

struct ArrActions {
    arr: Vec<usize>,

    /// Per-index "marked colored this frame" boolean. Doubles as the dedup
    /// flag for `color_indices`.
    color: Vec<bool>,
    color_indices: Vec<usize>,
    /// Indices that were colored last frame — re-marked dirty so the
    /// transition back to uncoloured is repainted.
    prev_color_indices: Vec<usize>,

    /// Indices touched this frame (write or color). Deduped via `dirty_flag`.
    dirty: Vec<usize>,
    dirty_flag: Vec<bool>,

    /// Pixel column → index range mapping. `col_indices_start[x]..col_indices_end[x]`
    /// is the half-open index range owned by pixel column `x`.
    col_indices_start: Vec<u32>,
    col_indices_end: Vec<u32>,
    /// Index → pixel column range mapping. `index_pixels_start[i]..index_pixels_end[i]`
    /// is the half-open pixel-column range that index `i` contributes to.
    index_pixels_start: Vec<u32>,
    index_pixels_end: Vec<u32>,

    /// Pixel columns flagged for repaint this frame, deduped via `dirty_col_flag`.
    dirty_cols: Vec<u32>,
    dirty_col_flag: Vec<bool>,

    /// Scratch buffers reused across `redraw_pixel_col` calls. Sized to
    /// `view.height + 1`. `diff_total[y]` = number of bars whose top row is
    /// exactly `y`; prefix sum gives "bars covering row y".
    diff_total: Vec<u32>,
    diff_colored: Vec<u32>,

    view: SubImg,
    name: usize,
    min: f64,
    max: f64,
}

impl Hash for ArrActions {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl ArrActions {
    fn new(arr: Vec<usize>, view: SubImg, name: usize) -> ArrActions {
        let min = *arr.iter().min().unwrap_or(&0);
        let max = *arr.iter().max().unwrap_or(&0);
        let len = arr.len();
        let mut a = ArrActions {
            arr,
            color: vec![false; len],
            color_indices: Vec::new(),
            prev_color_indices: Vec::new(),
            dirty: Vec::new(),
            dirty_flag: vec![false; len],
            col_indices_start: Vec::new(),
            col_indices_end: Vec::new(),
            index_pixels_start: vec![0; len],
            index_pixels_end: vec![0; len],
            dirty_cols: Vec::new(),
            dirty_col_flag: Vec::new(),
            diff_total: Vec::new(),
            diff_colored: Vec::new(),
            view: SubImg { x: 0, y: 0, width: 0, height: 0 },
            name,
            min: min as f64,
            max: max as f64,
        };
        a.set_view(view);
        a
    }

    /// Reassign the view rectangle and rebuild all pixel↔index mappings and
    /// scratch buffers. Caller is responsible for repainting (`force_full_redraw`).
    fn set_view(&mut self, view: SubImg) {
        self.view = view;
        let n = self.arr.len();
        let big_w = view.width as u64;
        let big_h = view.height as usize;

        // Per-pixel-column buffers.
        self.col_indices_start = vec![0u32; view.width as usize];
        self.col_indices_end = vec![0u32; view.width as usize];
        self.dirty_col_flag = vec![false; view.width as usize];
        self.dirty_cols.clear();

        // Per-row scratch buffers.
        self.diff_total = vec![0u32; big_h + 1];
        self.diff_colored = vec![0u32; big_h + 1];

        // Per-index buffers.
        self.index_pixels_start = vec![0u32; n];
        self.index_pixels_end = vec![0u32; n];

        if n == 0 || big_w == 0 {
            return;
        }

        if (n as u64) <= big_w {
            // n <= W: each index owns a span of pixels; each pixel has exactly
            // one owning index.
            for i in 0..n {
                let s = (i as u64 * big_w / n as u64) as u32;
                let e = ((i as u64 + 1) * big_w / n as u64) as u32;
                self.index_pixels_start[i] = s;
                self.index_pixels_end[i] = e;
                for x in s..e {
                    self.col_indices_start[x as usize] = i as u32;
                    self.col_indices_end[x as usize] = (i + 1) as u32;
                }
            }
        } else {
            // n > W: each pixel owns multiple indices; each index belongs to
            // exactly one pixel.
            for x in 0..view.width {
                let a = (x as u64 * n as u64 / big_w) as u32;
                let mut e = ((x as u64 + 1) * n as u64 / big_w) as u32;
                if e <= a {
                    e = a + 1;
                }
                if e as usize > n {
                    e = n as u32;
                }
                self.col_indices_start[x as usize] = a;
                self.col_indices_end[x as usize] = e;
                for i in a..e {
                    self.index_pixels_start[i as usize] = x;
                    self.index_pixels_end[i as usize] = x + 1;
                }
            }
        }
    }

    #[inline]
    fn mark_dirty(&mut self, i: usize) {
        if !self.dirty_flag[i] {
            self.dirty_flag[i] = true;
            self.dirty.push(i);
        }
    }

    #[inline]
    fn mark_color(&mut self, i: usize) {
        if !self.color[i] {
            self.color[i] = true;
            self.color_indices.push(i);
        }
    }

    /// Collapse the per-index dirty/color sets onto pixel columns and repaint
    /// every column that has any contributing index changed.
    fn finalize_frame(&mut self, fb: &mut Framebuffer) {
        // Build the per-index candidate set: dirty ∪ color_indices ∪ prev_color_indices.
        for k in 0..self.color_indices.len() {
            let i = self.color_indices[k];
            if !self.dirty_flag[i] {
                self.dirty_flag[i] = true;
                self.dirty.push(i);
            }
        }
        for k in 0..self.prev_color_indices.len() {
            let i = self.prev_color_indices[k];
            if !self.dirty_flag[i] {
                self.dirty_flag[i] = true;
                self.dirty.push(i);
            }
        }

        // Promote each dirty index to its pixel-column range.
        for k in 0..self.dirty.len() {
            let i = self.dirty[k];
            let s = self.index_pixels_start[i] as usize;
            let e = self.index_pixels_end[i] as usize;
            for x in s..e {
                if !self.dirty_col_flag[x] {
                    self.dirty_col_flag[x] = true;
                    self.dirty_cols.push(x as u32);
                }
            }
        }

        // Repaint every dirty pixel column.
        for k in 0..self.dirty_cols.len() {
            let x = self.dirty_cols[k];
            self.redraw_pixel_col(fb, x);
        }

        // Reset per-pixel-column flags.
        for k in 0..self.dirty_cols.len() {
            self.dirty_col_flag[self.dirty_cols[k] as usize] = false;
        }
        self.dirty_cols.clear();

        // Reset per-index dirty flags.
        for k in 0..self.dirty.len() {
            self.dirty_flag[self.dirty[k]] = false;
        }
        self.dirty.clear();

        // Clear `color` for next frame, then carry this frame's coloured
        // indices into prev_color_indices for transition tracking.
        for k in 0..self.color_indices.len() {
            self.color[self.color_indices[k]] = false;
        }
        std::mem::swap(&mut self.color_indices, &mut self.prev_color_indices);
        self.color_indices.clear();
    }

    /// Repaint every pixel column. Used after `set_view` since the cached
    /// framebuffer state no longer reflects the screen.
    fn force_full_redraw(&mut self, fb: &mut Framebuffer) {
        for x in 0..self.view.width {
            self.redraw_pixel_col(fb, x);
        }
    }

    /// Rasterise a single pixel column using density blending across the bars
    /// owned by that column. Cost: `O(k + H)` per call.
    fn redraw_pixel_col(&mut self, fb: &mut Framebuffer, x: u32) {
        let h = self.view.height;
        if h == 0 {
            return;
        }
        let stride = fb.width as usize * 3;
        let off_base = self.view.y as usize * stride + (self.view.x + x) as usize * 3;

        let i_start = self.col_indices_start[x as usize] as usize;
        let i_end = self.col_indices_end[x as usize] as usize;
        let k = i_end - i_start;

        if k == 0 {
            // No bars in this column (can happen for n > W on the very edge).
            // Wipe to black.
            for y in 0..h as usize {
                let off = off_base + y * stride;
                fb.data[off] = 0;
                fb.data[off + 1] = 0;
                fb.data[off + 2] = 0;
            }
            return;
        }

        // Reset diff scratch buffers up to row h (top of bars never goes
        // beyond `h` since we clamp bar_h to `h`).
        let h_plus_1 = (h + 1) as usize;
        self.diff_total[..h_plus_1].fill(0);
        self.diff_colored[..h_plus_1].fill(0);

        // For each bar in this column, mark its top row in the diff buffers.
        // Walking diff_total down the column gives the running cover-count.
        if self.max > 0.0 {
            for i in i_start..i_end {
                let bar_h = ((self.arr[i] as f64 / self.max) * h as f64) as u32;
                let bar_h = bar_h.min(h);
                if bar_h > 0 {
                    let top_row = (h - bar_h) as usize;
                    self.diff_total[top_row] += 1;
                    if self.color[i] {
                        self.diff_colored[top_row] += 1;
                    }
                }
            }
        }

        // Walk rows top to bottom, blending each pixel from the running counts.
        // Pixel = (white_count * WHITE + colored_count * GREEN) / k.
        let k_u32 = k as u32;
        let mut total: u32 = 0;
        let mut colored: u32 = 0;
        for y in 0..h as usize {
            total += self.diff_total[y];
            colored += self.diff_colored[y];
            let white = total - colored;
            let r = (WHITE[0] as u32 * white + GREEN[0] as u32 * colored) / k_u32;
            let g = (WHITE[1] as u32 * white + GREEN[1] as u32 * colored) / k_u32;
            let b = (WHITE[2] as u32 * white + GREEN[2] as u32 * colored) / k_u32;
            let off = off_base + y * stride;
            fb.data[off] = r as u8;
            fb.data[off + 1] = g as u8;
            fb.data[off + 2] = b as u8;
        }
    }
}
