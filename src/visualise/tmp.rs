use crate::traits::log_traits::SortLog;
use nannou::prelude::*;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref DATA_SWAPS: Mutex<Vec<SortLog>> = Mutex::new(Vec::new());
    static ref DATA_INITIAL_ARR: Mutex<Vec<u64>> = Mutex::new(Vec::new());
}

pub fn main(log: Vec<SortLog>, initial_arr: &[u64]) {
    {
        // Initialize the global state
        let mut data = DATA_SWAPS.lock().unwrap();
        *data = log;
        let mut arr = DATA_INITIAL_ARR.lock().unwrap();
        *arr = initial_arr.to_vec();
    }

    nannou::app(model).update(update).run();
}

struct Model {
    _window: window::Id,
    data: Vec<SortLog>,
    arr: Vec<u64>,
    ind: usize,
}

fn model(app: &App) -> Model {
    let data = DATA_SWAPS.lock().unwrap();
    let arr = DATA_INITIAL_ARR.lock().unwrap();
    let _window = app.new_window().view(view).build().unwrap();
    Model {
        _window,
        data: (*data).to_vec(),
        arr: (*arr).to_vec(),
        ind: 0,
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    let action = &_model.data[_model.ind];
    match action {
        SortLog::Swap { name, ind_a, ind_b } => _model.arr.swap(*ind_a, *ind_b),
        _ => {}
    }
    _model.ind += 1;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(PLUM);

    let num_elements = model.arr.len();
    let window_rect = app.window_rect();
    let bar_width = window_rect.w() / num_elements as f32;

    for (i, &value) in model.arr.iter().enumerate() {
        let x = map_range(i, 0, num_elements, window_rect.left(), window_rect.right());
        let height = map_range(
            value,
            0,
            *model.arr.iter().max().unwrap_or(&1),
            0.0,
            window_rect.h(),
        );
        draw.rect()
            .x_y(x, 0.0)
            .w_h(bar_width, height)
            .color(STEELBLUE);
    }

    draw.to_frame(app, &frame).unwrap();
}
