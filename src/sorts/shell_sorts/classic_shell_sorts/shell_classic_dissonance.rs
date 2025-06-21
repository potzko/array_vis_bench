use crate::create_sort;

create_sort!(sort, "shell sort classic dissonance", "O(N^2)", false);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let mut jump = arr.len() / 2;
    while jump >= 1 {
        logger.mark(format!("jump = {}", jump));
        for i in jump..arr.len() {
            let temp = arr[i];
            let mut j = i;
            while j >= jump && logger.cmp_gt_data(arr, j - jump, temp) {
                logger.write(arr, j, j - jump);
                j -= jump;
            }
            logger.write_data(arr, j, temp);
        }
        jump /= 2;
    }
    // Final insertion sort pass
    for i in 1..arr.len() {
        let temp = arr[i];
        let mut j = i;
        while j > 0 && logger.cmp_gt_data(arr, j - 1, temp) {
            logger.write(arr, j, j - 1);
            j -= 1;
        }
        logger.write_data(arr, j, temp);
    }
}
