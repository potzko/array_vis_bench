use sort_logger::SortLogger;

pub struct BubbleSortRecursive;

impl BubbleSortRecursive {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if arr.len() < 2 {
            return;
        }
        for i in 1..arr.len() {
            logger.cond_swap_le(arr, i, i - 1);
        }
        let len = arr.len();
        Self::sort(&mut arr[..len - 1], logger);
    }
}

sort_registry_macro::sort_family! {
    type Sort = BubbleSortRecursive;
    name        = "bubble sort recursive";
    big_o       = "O(N^2)";
    stable      = true;
    direct_sort = true;
    path        = ["bubble sorts", "bubble sort recursive"];
}
