use crate::traits::log_traits::SortLog;
use nannou::prelude::*;

pub fn main(stuff: Vec<SortLog>) {
    nannou::app(|app| model(app, stuff)).update(update).run();
}

struct Model {
    _window: window::Id,
    data: Vec<SortLog>,
    ind: usize,
}

// Note the additional parameter for the initial data
fn model(app: &App, initial_data: Vec<SortLog>) -> Model {
    let _window = app.new_window().view(view).build().unwrap();
    Model {
        _window,
        data: initial_data,
        ind: 0, // Initialize `ind` to 0
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(PLUM);
    draw.ellipse().color(STEELBLUE);
    draw.to_frame(app, &frame).unwrap();
}
