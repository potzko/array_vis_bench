const MAX_SIZE: usize = 100000;
const BIG_O: &str = "O(N*log(N))";
const NAME: &str = "smooth sort";

use crate::traits;

pub struct HeapSort {}

impl traits::sort_traits::SortAlgo for HeapSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &'static str {
        BIG_O
    }
    fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
        &self,
        arr: &mut [T],
        logger: &mut U,
    ) {
        sort::<T, U>(arr, logger);
    }
    fn name(&self) -> &'static str {
        NAME
    }
}
use std::fmt::Debug;
#[allow(clippy::derivable_impls)]
impl Default for HeapSort {
    fn default() -> Self {
        HeapSort {}
    }
}
impl Debug for HeapSort {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Result::Ok(())
    }
}

const LEONARDO_NUMS: [usize; 43] = [
    1, 1, 3, 5, 9, 15, 25, 41, 67, 109, 177, 287, 465, 753, 1219, 1973, 3193, 5167, 8361, 13529,
    21891, 35421, 57313, 92735, 150049, 242785, 392835, 635621, 1028457, 1664079, 2692537, 4356617,
    7049155, 11405773, 18454929, 29860703, 48315633, 78176337, 126491971, 204668309, 331160281,
    535828591, 866988873,
]; //max u32 leonardo number

pub fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(input: &mut [T], logger: &mut U) {
    if input.len() < 2 {
        return;
    }

    let input = input;
    let in_len = input.len();
    let mut heap = Vec::<usize>::new();

    for i in 0..in_len {
        if heap.len() >= 2 && heap[heap.len() - 2] == heap[heap.len() - 1] + 1 {
            heap.pop();
            let len_leo = heap.len();
            heap[len_leo - 1] += 1;
        } else if !heap.is_empty() && heap[heap.len() - 1] == 1 {
            heap.push(0);
        } else {
            heap.push(1);
        }
        restore_heap(input, i, &heap, logger);
    }

    for i in (0..in_len).rev() {
        if heap[heap.len() - 1] < 2 {
            heap.pop();
        } else {
            let k = heap.pop().unwrap();
            let t = get_child_trees(i, k);
            // tr kr tl kl
            // 0  1  2  3
            heap.push(t[3]);
            restore_heap(input, t[2], &heap, logger);
            heap.push(t[1]);
            restore_heap(input, t[0], &heap, logger);
        }
    }
}

fn restore_heap<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    index: usize,
    heap: &Vec<usize>,
    logger: &mut U,
) {
    // Insertion sorting
    let mut current = heap.len() - 1;
    let mut i = index;
    let mut k = heap[current];

    while current > 0 {
        let j = i - LEONARDO_NUMS[k];
        if logger.cmp_gt(arr, j, i)
            && (k < 2 || logger.cmp_gt(arr, j, i - 1) && logger.cmp_gt(arr, j, i - 2))
        {
            logger.swap(arr, i, j);
            i = j;
            current -= 1;
            k = heap[current];
        } else {
            break;
        }
    }

    while k >= 2 {
        let t = get_child_trees(i, k);
        // tr kr tl kl
        // 0  1  2  3
        if logger.cmp_lt(arr, i, t[0]) || logger.cmp_lt(arr, i, t[2]) {
            if logger.cmp_gt(arr, t[0], t[2]) {
                logger.swap(arr, i, t[0]);
                i = t[0];
                k = t[1];
            } else {
                logger.swap(arr, i, t[2]);
                i = t[2];
                k = t[3];
            }
        } else {
            break;
        }
    }
}

fn get_child_trees(i: usize, k: usize) -> [usize; 4] {
    let tr = i - 1;
    let kr = k - 2;
    let tl = tr - LEONARDO_NUMS[kr];
    let kl = k - 1;
    [tr, kr, tl, kl]
}
