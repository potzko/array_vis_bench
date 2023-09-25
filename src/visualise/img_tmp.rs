#![allow(dead_code)]

extern crate ffmpeg_next as ffmpeg;
extern crate image;

use super::sub_image::SubImg;
use crate::traits::log_traits::SortLog;
use image::{GenericImage, ImageBuffer, Rgba};
use std::process::Command;

const ACTIONS_PER_FRAME: usize = 200;

pub fn main(arr: &mut [u64], actions: &[SortLog<u64>]) {
    let script_path = r"D:\Programing\IDE\vscProjects\rustFolder\array_vis_bench\src\python_scripts\empty_tmp_folder.py";
    Command::new("python").arg(script_path).status().unwrap();

    let width: u32 = arr.len() as u32;
    let height = (width as f64 / 16.0 * 9.0) as u32;
    let min_max = (*arr.iter().min().unwrap(), *arr.iter().max().unwrap());

    let color = Rgba([0xff, 0xff, 0xff, 0xff]);
    let bg_color: Rgba<u8> = Rgba([0x0, 0x0, 0x0, 0xff]);
    println!(
        "{} frames to generate",
        actions.len() / ACTIONS_PER_FRAME + 3
    );
    let mut img = ImageBuffer::from_fn(width, height, |_, _| Rgba([0x00, 0x00, 0x00, 0xff]));
    let view = SubImg {
        x: 0,
        y: 0,
        width,
        height,
    };
    full_rander_vec(&mut img, &view, arr, min_max, bg_color, color);

    let mut redraw = vec![false; arr.len()];
    let mut recolor = vec![false; arr.len()];
    let mut old_color = vec![false; arr.len()];
    for i in 0..actions.len() / ACTIONS_PER_FRAME + 3 {
        if i % 100 == 0 {
            println!("frame {} of {}", i, actions.len() / ACTIONS_PER_FRAME + 3);
        }
        update(arr, actions, i, &mut redraw, &mut recolor);
        img.save(format!(
            "D:\\Programing\\IDE\\vscProjects\\rustFolder\\array_vis_bench\\tmp\\{}.png",
            i
        ))
        .unwrap();
        for i in 0..arr.len() {
            redraw[i] = (redraw[i] | old_color[i]) && !recolor[i];
        }
        update_image(&mut img, &view, arr, min_max, bg_color, color, &redraw);
        update_image(
            &mut img,
            &view,
            arr,
            min_max,
            bg_color,
            Rgba([0x0, 0xa0, 0x60, 0xff]),
            &recolor,
        );
        old_color = recolor;
        redraw = vec![false; arr.len()];
        recolor = vec![false; arr.len()];
    }

    // Construct the path to the Python script
    let script_path = r"D:\Programing\IDE\vscProjects\rustFolder\array_vis_bench\src\python_scripts\pngs_to_vid.py";
    Command::new("python").arg(script_path).status().unwrap();
    let script_path = r"D:\Programing\IDE\vscProjects\rustFolder\array_vis_bench\src\python_scripts\empty_tmp_folder.py";
    Command::new("python").arg(script_path).status().unwrap();
}

fn full_rander_vec(
    img: &mut impl GenericImage<Pixel = Rgba<u8>>,
    view: &SubImg,
    arr: &[u64],
    min_max: (u64, u64),
    color: Rgba<u8>,
    color_bg: Rgba<u8>,
) {
    update_image(
        img,
        view,
        arr,
        min_max,
        color,
        color_bg,
        &vec![true; arr.len()],
    );
}

fn update_image(
    img: &mut impl GenericImage<Pixel = Rgba<u8>>,
    view: &SubImg,
    arr: &[u64],
    min_max: (u64, u64),
    color: Rgba<u8>,
    color_bg: Rgba<u8>,
    draw_inds: &[bool],
) {
    for (ind, val) in draw_inds.iter().enumerate() {
        if *val {
            view.rect(
                img,
                ind as u32 * (view.width / arr.len() as u32),
                0,
                view.width / arr.len() as u32,
                view.height - ((arr[ind] as f64 / min_max.1 as f64) * view.height as f64) as u32,
                color,
            );
            view.rect(
                img,
                ind as u32 * (view.width / arr.len() as u32),
                view.height - ((arr[ind] as f64 / min_max.1 as f64) * view.height as f64) as u32,
                view.width / arr.len() as u32,
                ((arr[ind] as f64 / min_max.1 as f64) * view.height as f64) as u32,
                color_bg,
            );
        }
    }
}

fn update(
    arr: &mut [u64],
    actions: &[SortLog<u64>],
    iteration: usize,
    draw: &mut [bool],
    color: &mut [bool],
) {
    if iteration * ACTIONS_PER_FRAME >= actions.len() {
        return;
    }
    for action in actions[iteration * ACTIONS_PER_FRAME..]
        .iter()
        .take(ACTIONS_PER_FRAME)
    {
        match action {
            SortLog::Swap { name, ind_a, ind_b } => {
                arr.swap(*ind_a, *ind_b);
                draw[*ind_a] = true;
                draw[*ind_b] = true;
            }
            SortLog::WriteInArr { name, ind_a, ind_b } => {
                arr[*ind_a] = arr[*ind_b];
                draw[*ind_a] = true;
                draw[*ind_b] = true;
            }
            SortLog::WriteData { name, ind, data } => {
                arr[*ind] = *data;
                draw[*ind] = true;
            }
            SortLog::CmpInArr {
                name,
                ind_a,
                ind_b,
                result,
            } => {
                color[*ind_a] = true;
                color[*ind_b] = true;
            }
            SortLog::CmpData {
                name,
                ind,
                data,
                result,
            } => {
                color[*ind] = true;
            }
            _ => {}
        }
    }
}

fn set_bg(img: &mut impl GenericImage<Pixel = Rgba<u8>>, view: &SubImg, color: Rgba<u8>) {
    view.rect(img, 0, 0, view.width, view.height, color)
}
