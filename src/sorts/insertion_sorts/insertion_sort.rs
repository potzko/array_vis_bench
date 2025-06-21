use crate::create_sort;

create_sort!(sort, "insertion sort", "O(N^2)", true);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for i in 1..arr.len() {
        let num = arr[i];
        let mut ind = i;

        while ind > 0 && logger.cmp_gt_data(arr, ind - 1, num) {
            logger.write(arr, ind, ind - 1);
            ind -= 1;
        }
        logger.write_data(arr, ind, num);
    }
}
