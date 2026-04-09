pub fn is_sorted<T: Ord>(arr: &[T]) -> bool {
    arr.windows(2).all(|w| w[0] <= w[1])
}

pub fn is_sorted_arr<T: Ord>(arr: &[T], arr_original: &mut [T]) -> bool {
    arr_original.sort();
    arr_original == arr
}
