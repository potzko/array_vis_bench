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
