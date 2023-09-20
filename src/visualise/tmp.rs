use crate::traits::log_traits::SortLog;
use nannou::prelude::*;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref DATA_SWAPS: Mutex<Vec<SortLog<u64>>> = Mutex::new(Vec::new());
    static ref DATA_INITIAL_ARR: Mutex<Vec<u64>> = Mutex::new(Vec::new());
}

const ACTIONS_PER_FRAME: usize = 100;

pub fn main(log: Vec<SortLog<u64>>, initial_arr: &[u64]) {
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
    data: Vec<SortLog<u64>>,
    arr: Vec<u64>,
    ind: usize,
    draw: Vec<[usize; 2]>,
}

fn model(app: &App) -> Model {
    let data = DATA_SWAPS.lock().unwrap();
    let arr = DATA_INITIAL_ARR.lock().unwrap();
    app.set_loop_mode(LoopMode::NTimes {
        number_of_updates: data.len() / ACTIONS_PER_FRAME + 1,
    });
    let _window = app.new_window().view(view).build().unwrap();

    Model {
        _window,
        data: (*data).to_vec(),
        arr: (*arr).to_vec(),
        ind: 0,
        draw: Vec::<[usize; 2]>::new(),
    }
}

fn update(_app: &App, state: &mut Model, _update: Update) {
    for action in state.data[state.ind * ACTIONS_PER_FRAME..]
        .iter()
        .take(ACTIONS_PER_FRAME)
    {
        match action {
            SortLog::Swap { name, ind_a, ind_b } => {
                state.arr.swap(*ind_a, *ind_b);
                state.draw.push([*ind_a, *name]);
                state.draw.push([*ind_b, *name]);
            }
            SortLog::WriteInArr { name, ind_a, ind_b } => {
                state.arr[*ind_a] = state.arr[*ind_b];
                state.draw.push([*ind_a, *name]);
                state.draw.push([*ind_b, *name]);
            }
            SortLog::WriteData { name, ind, data } => {
                state.arr[*ind] = *data;
                state.draw.push([*ind, *name]);
            }
            _ => {}
        }
    }
    state.ind += 1;
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
