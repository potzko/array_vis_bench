use crate::create_sort;

create_sort!(sort, "shell sort sedgewick", "O(N^(4/3))", false);

#[inline]
fn sequence_sedgewick(iter: usize) -> usize {
    if iter == 0 {
        return 1;
    }
    4_usize.pow(iter as u32) + 3 * 2_usize.pow(iter as u32 - 1) + 1
}

fn vec_sedgewick(len: usize) -> Vec<usize> {
    let mut ret = Vec::new();
    let mut iter = 0;
    loop {
        let num = sequence_sedgewick(iter);
        if num >= len {
            break;
        }
        ret.push(num);
        iter += 1
    }
    ret
}

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let len = arr.len();
    let jumps = vec_sedgewick(len);
    for &jump in jumps.iter().rev() {
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
}
