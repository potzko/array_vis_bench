use crate::create_sort;
use crate::traits::sort_traits::SortAlgo;

create_sort!(sort, "shell sort optimized 256 elements", "O(N^1.5)", false);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for jump in [84, 25] {
        logger.mark_mssg(&format!("jump = {}", jump));
        for i in jump..arr.len() {
            let temp = arr[i];
            let mut j = i;
            while j >= jump && logger.cmp_gt_data(arr, j - jump, temp) {
                logger.write(arr, j, j - jump);
                j -= jump;
            }
            logger.write_data(arr, j, temp);
        }
    }
    type SmallSort<A, B> = crate::sorts::insertion_sorts::insertion_sort::SortImp<A, B>;
    SmallSort::sort(arr, logger);
}
