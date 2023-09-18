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
