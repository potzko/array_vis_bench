use crate::traits::log_traits::VisualizerLogger;
use sort_vis::{Mp4Config, Mp4Visualizer, Pacing, Visualizer};

pub fn visualise_sort(
    arr: &mut [usize],
    logger: &mut VisualizerLogger<usize>,
    choice: &[String],
    duration_secs: Option<f64>,
) {
    let mut original_arr = Vec::with_capacity(arr.len());
    arr.clone_into(&mut original_arr);
    let values = crate::sorts::fn_sort(
        arr,
        logger as &mut dyn crate::traits::log_traits::SortLogger<usize>,
        choice,
    );

    let config = match duration_secs {
        Some(s) => Mp4Config { pacing: Pacing::DurationSeconds(s), ..Default::default() },
        None => Mp4Config::default(),
    };
    let mut viz = Mp4Visualizer::new(config);
    viz.render(&original_arr, arr.as_ptr() as usize, &logger.log);

    println!("{:?}", values);
}
