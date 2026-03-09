#![allow(dead_code)]
extern crate image;

use crate::sub_image::SubImg;
use sort_logger::SortLog;
use image::{DynamicImage, GenericImage, ImageBuffer, Rgba};
use std::hash::Hash;
use std::io::Write;
use std::mem::size_of;
use std::process::{Child, ChildStdin, Command, Stdio};

const ACTIONS_PER_FRAME: usize = 100;
const OUTPUT_WIDTH: u32 = 1920;
const OUTPUT_HEIGHT: u32 = 1080;
const FRAMERATE: u32 = 30;

/// Switch encoding target by uncommenting one line:
// const ENCODING: Encoding = Encoding::Lossless;
// const ENCODING: Encoding = Encoding::Lossy;
const ENCODING: Encoding = Encoding::Fast;

/// Lossless — perfect quality, large file, moderate encode speed.
/// Lossy     — good quality (CRF 23), small file, moderate encode speed.
/// Fast      — lower quality (CRF 28), small file, fastest encode.
enum Encoding {
    Lossless,
    Lossy,
    Fast,
}

const WHITE: Rgba<u8> = Rgba([0xff, 0xff, 0xff, 0xff]);
const BLACK: Rgba<u8> = Rgba([0x0, 0x0, 0x0, 0xff]);
const GREEN: Rgba<u8> = Rgba([0x0, 0xa0, 0x60, 0xff]);
const BLUE: Rgba<u8> = Rgba([0x0, 0x30, 0xff, 0xff]);

enum VisualAction {
    Draw,
    Color,
}

fn get_views(view: &SubImg, amount: u32) -> Vec<SubImg> {
    let mut ret: Vec<SubImg> = Vec::with_capacity(amount as usize);
    let hight = view.height / amount;
    for i in 0..amount {
        ret.push(view.make_sub_img(0, i * hight, view.width, hight));
    }
    ret
}

fn spawn_ffmpeg() -> Child {
    let video_size = format!("{}x{}", OUTPUT_WIDTH, OUTPUT_HEIGHT);
    let framerate = FRAMERATE.to_string();

    // Args shared by all modes: input spec
    let mut args: Vec<&str> = vec![
        "-y",
        "-f", "rawvideo",
        "-pixel_format", "rgb24",
        "-video_size", &video_size,
        "-framerate", &framerate,
        "-i", "pipe:0",
    ];

    // Mode-specific output args
    match ENCODING {
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

    args.push("output.mp4");

    Command::new("ffmpeg")
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to spawn ffmpeg — is it installed?")
}

fn push_frame(stdin: &mut ChildStdin, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) {
    let resized = DynamicImage::ImageRgba8(image.clone())
        .resize_exact(OUTPUT_WIDTH, OUTPUT_HEIGHT, image::imageops::FilterType::Nearest)
        .to_rgb8();
    stdin.write_all(resized.as_raw()).unwrap();
}

pub fn render_gif(arr: &[usize], name: usize, actions: &[SortLog<usize>]) {
    let arr = arr.to_vec();

    let mut inplace = true;
    let mut arrs: Vec<(usize, usize)> = Vec::new();
    let mut ind_arrs: Vec<(usize, usize)> = Vec::new();
    for i in actions {
        match i {
            SortLog::CreateAuxArr { name, length } => {
                ind_arrs.push((*name, *length));
                inplace = false;
            }
            SortLog::CreateAuxArrT { name, length } => {
                arrs.push((*name, *length));
                inplace = false;
            }
            _ => {}
        }
    }

    let width: u32 = arr.len() as u32;
    let height = (width as f64 / 16.0 * 9.0) as u32;
    println!(
        "{} frames to generate",
        actions.len() / ACTIONS_PER_FRAME + 3
    );
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_fn(width, height, |_, _| Rgba::<u8>([0x00, 0x00, 0x00, 0xff]));

    let mut arrs: Vec<ArrActions> = Vec::new();
    let view = SubImg {
        x: 0,
        y: if inplace { 0 } else { height / 2 },
        width,
        height: if inplace { height } else { height / 2 },
    };
    let aux_view = SubImg {
        x: 0,
        y: 0,
        width,
        height: height / 2,
    };
    arrs.push(ArrActions::new(arr, view, name));

    let mut ffmpeg = spawn_ffmpeg();
    let stdin = ffmpeg.stdin.as_mut().expect("ffmpeg stdin not available");

    let mut i = 1;
    while i * ACTIONS_PER_FRAME - ACTIONS_PER_FRAME < actions.len() {
        let mut split_points: Vec<usize> = vec![i * ACTIONS_PER_FRAME - ACTIONS_PER_FRAME];
        #[allow(clippy::needless_range_loop)]
        for ii in i * ACTIONS_PER_FRAME - ACTIONS_PER_FRAME
            ..std::cmp::min(i * ACTIONS_PER_FRAME, actions.len())
        {
            match actions[ii] {
                SortLog::CreateAuxArr { .. } => split_points.push(ii),
                SortLog::CreateAuxArrT { .. } => {
                    split_points.push(ii);
                }
                _ => {}
            }
        }
        for ii in 1..split_points.len() {
            for arr_view in arrs.iter_mut() {
                arr_view.update(&actions[split_points[ii - 1]..split_points[ii]], &mut img)
            }
            push_frame(stdin, &img);

            match actions[split_points[ii]] {
                SortLog::CreateAuxArrT { name, length }
                | SortLog::CreateAuxArr { name, length } => {
                    aux_view.rect(&mut img, 0, 0, aux_view.width, aux_view.height, BLACK);
                    let views = get_views(&aux_view, arrs.len() as u32);
                    arrs.push(ArrActions::new(
                        vec![0; length],
                        SubImg {
                            x: 0,
                            y: 0,
                            width: 1000,
                            height: 000,
                        },
                        name,
                    ));
                    let len = arrs.len();
                    for iii in 1..len {
                        arrs[iii].view = views[iii - 1];
                        arrs[iii].full_rander_vec(&mut img, BLACK, WHITE)
                    }
                }
                SortLog::FreeAuxArr { name: _ } => {}
                _ => {}
            }
        }

        for arr_view in arrs.iter_mut() {
            arr_view.update(
                &actions[*split_points.last().unwrap()
                    ..std::cmp::min(i * ACTIONS_PER_FRAME, actions.len())],
                &mut img,
            )
        }
        push_frame(stdin, &img);

        i += 1;
        if i % 100 == 0 {
            println!("{i} of {}", actions.len() / ACTIONS_PER_FRAME + 3);
        }
    }

    // Close stdin so ffmpeg knows the stream is done
    drop(ffmpeg.stdin.take());
    let status = ffmpeg.wait().expect("failed to wait on ffmpeg");
    if !status.success() {
        eprintln!("ffmpeg exited with status: {status}");
    }
}

struct ArrActions {
    arr: Vec<usize>,
    color: Vec<bool>,
    old_color: Vec<bool>,
    draw: Vec<bool>,
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
    fn update(
        &mut self,
        actions: &[SortLog<usize>],
        img: &mut impl GenericImage<Pixel = Rgba<u8>>,
    ) {
        let size_t = size_of::<usize>();
        for action in actions {
            match action {
                SortLog::Swap { name, ind_a, ind_b } => {
                    if *name >= self.name && *name < self.name + self.arr.len() * size_t {
                        let adjusted_ind_a = ind_a + (name - self.name) / size_t;
                        let adjusted_ind_b = ind_b + (name - self.name) / size_t;
                        self.arr.swap(adjusted_ind_a, adjusted_ind_b);
                        self.draw[adjusted_ind_a] = true;
                        self.draw[adjusted_ind_b] = true;
                    }
                }
                SortLog::WriteDataU { name, ind, data } => {
                    if *name >= self.name && *name < self.name + self.arr.len() * size_t {
                        let adjusted_ind = ind + (name - self.name) / size_t;
                        self.arr[adjusted_ind] = *data;
                        self.draw[adjusted_ind] = true;

                        let data_f64 = *data as f64;
                        if self.min > data_f64 {
                            self.min = data_f64
                        }
                        if self.max < data_f64 {
                            self.max = data_f64
                        }
                    }
                }
                SortLog::WriteInArr { name, ind_a, ind_b } => {
                    if *name >= self.name && *name < self.name + self.arr.len() * size_t {
                        let adjusted_ind_a = ind_a + (name - self.name) / size_t;
                        let adjusted_ind_b = ind_b + (name - self.name) / size_t;
                        self.arr[adjusted_ind_a] = self.arr[adjusted_ind_b];
                        self.draw[adjusted_ind_a] = true;
                        self.draw[adjusted_ind_b] = true;
                    }
                }
                SortLog::WriteData { name, ind, data } => {
                    if *name >= self.name && *name < self.name + self.arr.len() * size_t {
                        let adjusted_ind = ind + (name - self.name) / size_t;
                        self.arr[adjusted_ind] = *data;
                        self.draw[adjusted_ind] = true;

                        let data_f64 = *data as f64;
                        if self.min > data_f64 {
                            self.min = data_f64
                        }
                        if self.max < data_f64 {
                            self.max = data_f64
                        }
                    }
                }
                SortLog::CmpInArr {
                    name,
                    ind_a,
                    ind_b,
                    result: _,
                } => {
                    if *name >= self.name && *name < self.name + self.arr.len() * size_t {
                        let adjusted_ind_a = ind_a + (name - self.name) / size_t;
                        let adjusted_ind_b = ind_b + (name - self.name) / size_t;
                        self.color[adjusted_ind_a] = true;
                        self.color[adjusted_ind_b] = true;
                    }
                }
                SortLog::CmpData {
                    name,
                    ind,
                    data: _,
                    result: _,
                } => {
                    if *name >= self.name && *name < self.name + self.arr.len() * size_t {
                        let adjusted_ind = ind + (name - self.name) / size_t;
                        self.color[adjusted_ind] = true;
                    }
                }
                SortLog::CmpDataU {
                    name,
                    ind,
                    data: _,
                    result: _,
                } => {
                    if *name >= self.name && *name < self.name + self.arr.len() * size_t {
                        let adjusted_ind = ind + (name - self.name) / size_t;
                        self.color[adjusted_ind] = true;
                    }
                }
                SortLog::CmpAcrossArrs {
                    name_a,
                    ind_a,
                    name_b,
                    ind_b,
                    result: _,
                } => {
                    if *name_a >= self.name && *name_a < self.name + self.arr.len() * size_t {
                        let adjusted_ind = ind_a + (name_a - self.name) / size_t;
                        self.color[adjusted_ind] = true;
                    }
                    if *name_b >= self.name && *name_b < self.name + self.arr.len() * size_t {
                        let adjusted_ind = ind_b + (name_b - self.name) / size_t;
                        self.color[adjusted_ind] = true;
                    }
                }
                _ => {}
            }
        }
        for i in 0..self.arr.len() {
            self.draw[i] = (self.draw[i] | self.old_color[i]) && !self.color[i];
        }

        self.update_arr_view(img, BLACK, WHITE, &self.draw.clone());
        self.update_arr_view(img, BLACK, GREEN, &self.color.clone());

        self.old_color = self.color.clone();
        self.draw = vec![false; self.arr.len()];
        self.color = vec![false; self.arr.len()];
    }
    fn update_arr_view(
        &mut self,
        img: &mut impl GenericImage<Pixel = Rgba<u8>>,
        color_bg: Rgba<u8>,
        color: Rgba<u8>,
        draw_inds: &[bool],
    ) {
        for (ind, val) in draw_inds.iter().enumerate() {
            if *val {
                self.view.rect(
                    img,
                    ind as u32 * (self.view.width / self.arr.len() as u32),
                    0,
                    self.view.width / self.arr.len() as u32,
                    self.view.height
                        - ((self.arr[ind] as f64 / self.max) * self.view.height as f64) as u32,
                    color_bg,
                );
                self.view.rect(
                    img,
                    ind as u32 * (self.view.width / self.arr.len() as u32),
                    self.view.height
                        - ((self.arr[ind] as f64 / self.max) * self.view.height as f64) as u32,
                    self.view.width / self.arr.len() as u32,
                    ((self.arr[ind] as f64 / self.max) * self.view.height as f64) as u32,
                    color,
                );
            }
        }
    }

    fn full_rander_vec(
        &mut self,
        img: &mut impl GenericImage<Pixel = Rgba<u8>>,
        color: Rgba<u8>,
        color_bg: Rgba<u8>,
    ) {
        self.update_arr_view(img, color, color_bg, &vec![true; self.arr.len()]);
    }

    fn new(arr: Vec<usize>, view: SubImg, name: usize) -> ArrActions {
        let min = *arr.iter().min().unwrap_or(&0);
        let max = *arr.iter().max().unwrap_or(&0);
        let len = arr.len();
        ArrActions {
            arr,
            color: vec![false; len],
            old_color: vec![false; len],
            draw: vec![true; len],
            view,
            name,
            min: min as f64,
            max: max as f64,
        }
    }
}
