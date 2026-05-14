use crate::traits::log_traits::{SortLogger, VisualizerLogger};
use sort_logger::SortLog;
use sort_vis::{Mp4Config, Mp4Visualizer, Visualizer};

pub fn visualise_sort(
    arr: &mut [usize],
    logger: &mut VisualizerLogger<usize>,
    choice: &[String],
    config: Mp4Config,
) {
    // Drive the input array's creation through the same logger events any
    // other array uses: one CreateAuxArrT for the slice's pointer, then
    // one WriteData per element to populate. The visualiser then needs no
    // back-channel — the log alone describes the full timeline including
    // the starting state.
    logger.log_aux_arr_t(arr);
    let name = arr.as_ptr() as usize;
    for (i, &v) in arr.iter().enumerate() {
        logger.log(SortLog::WriteData { name, ind: i, data: v });
    }

    let values = crate::sorts::fn_sort(arr, logger as &mut dyn SortLogger<usize>, choice);

    let mut viz = Mp4Visualizer::new(config);
    viz.render(&logger.log);

    println!("{:?}", values);
}
