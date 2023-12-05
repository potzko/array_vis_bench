pub mod img_tmp;
pub mod sub_image;
use crate::traits::log_traits::VisualizerLogger;

pub fn visualise_sort(arr: &mut [usize], logger: &mut VisualizerLogger<usize>, choice: &[String]) {
    let mut original_arr = Vec::with_capacity(arr.len());
    arr.clone_into(&mut original_arr);
    let values = crate::sorts::fn_sort(arr, logger, choice);
    img_tmp::main(&original_arr, arr.as_ptr() as usize, &logger.log);
    println!("{:?}", values);
}
