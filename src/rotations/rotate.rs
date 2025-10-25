use crate::traits::log_traits::SortLogger;

trait RotateImpl<T: Ord + Copy, U SortLogger<T>> {
    fn rotate(arr: &mut [T], split_ind: usize, logger: &mut U);
}


#[allow(dead_code)]
pub fn rotate_2<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], split_ind: usize, logger: &mut U) {
    //println!("{}, {}", arr.len(), split_ind);
    let len = arr.len();
    if split_ind == 0 {
        return;
    }
    for i in split_ind..len {
        logger.swap(arr, i - split_ind, i);
    }
    rotate(
        &mut arr[len - len % split_ind..],
        split_ind % len % split_ind,
        logger,
    );
}

struct Reverse3TimeRotatation<T: Ord + Copy, U: SortLogger<T>> RotateImpl<T, U>;
impl<T: Ord + Copy, U: SortLogger<T>> RotateImpl<T, U> for Reverse3TimeRotatation {
    pub fn rotate(arr: &mut [T], split_ind: usize, logger: &mut U) {
        reverse(&mut arr[..split_ind], logger);
        reverse(&mut arr[split_ind..], logger);
        reverse(arr, logger);
    }
}

#[allow(dead_code)]
fn reverse<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for i in 0..arr.len() / 2 {
        logger.swap(arr, i, arr.len() - i - 1);
    }
}

