use crate::sorts::fn_sort;
use crate::traits::log_traits::*;
use crate::utils::array_gen;

pub fn is_sorted<T: Ord>(arr: &[T]) -> bool {
    if arr.len() < 2 {
        return true;
    }
    for i in 1..arr.len() {
        if arr[i] < arr[i - 1] {
            return false;
        }
    }
    true
}

pub fn is_sorted_arr<T: Ord>(arr: &[T], arr_original: &mut [T]) -> bool {
    arr_original.sort();
    arr_original == arr
}

pub fn test_sort(choice: &[String]) -> bool {
    test_sort_small_arrs(choice)
}
pub fn test_sort_small_arrs(choice: &[String]) -> bool {
    let mut flag = true;
    for i in 0..=46655 {
        let mut i = i;
        let mut arr = [0; 6];
        let mut arr_tmp = [0; 6];
        for ii in 0..6 {
            arr[ii] = i % 6;
            arr_tmp[ii] = i % 6;
            i /= 6;
        }
        fn_sort(&mut arr, &mut (), choice);
        flag = flag && is_sorted(&arr);
    }
    flag
}

struct UnstableNum {
    a: usize,
    b: usize,
}
impl PartialEq for UnstableNum {
    fn eq(&self, other: &Self) -> bool {
        self.a.eq(&other.a)
    }
}
impl PartialOrd for UnstableNum {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.a.partial_cmp(&other.a)
    }
}
impl Eq for UnstableNum {}
impl Ord for UnstableNum {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.a.cmp(&other.a)
    }
}
