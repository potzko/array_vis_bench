use crate::create_sort;

create_sort!(sort, "shell sort knuth", "O(N^(3/2))", false);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let mut k = 1;
    while k < arr.len() {
        k = 3 * k + 1;
    }
    k = (k - 1) / 3;

    while k >= 1 {
        logger.mark(format!("jump = {}", k));
        for i in k..arr.len() {
            let temp = arr[i];
            let mut j = i;
            while j >= k && logger.cmp_gt_data(arr, j - k, temp) {
                logger.write(arr, j, j - k);
                j -= k;
            }
            logger.write_data(arr, j, temp);
        }
        k = (k - 1) / 3;
    }
}
