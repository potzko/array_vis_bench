use sort_logger::SortLogger;

pub struct SlowSortPotzko;

impl SlowSortPotzko {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        slow_sort_rec::<T, U>(arr, logger);
    }
}

fn slow_sort_rec<T, U>(arr: &mut [T], logger: &mut U)
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
{
    if arr.len() < 2 {
        return;
    }
    self::slow_sort_rec::<T, U>(&mut arr[1..], logger);
    logger.cond_swap_gt(arr, 0, 1);
    self::slow_sort_rec::<T, U>(&mut arr[1..], logger);
}

sort_registry_macro::sort_family! {
    type Sort = SlowSortPotzko;
    name        = "slow sort potzko";
    big_o       = "O(N^logN)";
    stable      = false;
    direct_sort = true;
    path        = ["fun sorts", "slow sort potzko"];
    // T(n) = 2·T(n − 1) so it's effectively O(2^N) — n=20 takes ~1M
    // operations, n=24 is borderline. Cap aggressively.
    max_n_for_tests = 20;
}
