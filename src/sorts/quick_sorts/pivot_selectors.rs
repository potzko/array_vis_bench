use crate::traits::log_traits::SortLogger;

pub trait PivotSelector {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], logger: &mut U) -> usize;
}

pub struct FirstElement;
pub struct MiddleElement;
pub struct LastElement;
pub struct MedianOfThree;
pub struct MedianOfMedians;
pub struct Ninther;

impl PivotSelector for FirstElement {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        _arr: &[T],
        _logger: &mut U,
    ) -> usize {
        0
    }
}

impl PivotSelector for MiddleElement {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        _logger: &mut U,
    ) -> usize {
        arr.len() / 2
    }
}

impl PivotSelector for LastElement {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        _logger: &mut U,
    ) -> usize {
        arr.len() - 1
    }
}

impl PivotSelector for MedianOfThree {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
    ) -> usize {
        median_index(arr, logger, 0, arr.len() / 2, arr.len() - 1)
    }
}

impl PivotSelector for MedianOfMedians {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
    ) -> usize {
        let len = arr.len();
        if len < 5 {
            return len / 2;
        }
        let samples = [0, len / 4, len / 2, (3 * len) / 4, len - 1];
        let m1 = median_index(arr, logger, samples[0], samples[1], samples[2]);
        let m2 = median_index(arr, logger, samples[2], samples[3], samples[4]);
        median_index(arr, logger, m1, samples[2], m2)
    }
}

impl PivotSelector for Ninther {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
    ) -> usize {
        let len = arr.len();
        if len < 9 {
            return median_index(arr, logger, 0, len / 2, len - 1);
        }
        // 9 evenly spaced samples, grouped into 3 triples
        let s = [
            0, len / 8, len / 4,              // Group A
            3 * len / 8, len / 2, 5 * len / 8, // Group B
            3 * len / 4, 7 * len / 8, len - 1, // Group C
        ];
        let m1 = median_index(arr, logger, s[0], s[1], s[2]);
        let m2 = median_index(arr, logger, s[3], s[4], s[5]);
        let m3 = median_index(arr, logger, s[6], s[7], s[8]);
        median_index(arr, logger, m1, m2, m3)
    }
}

/// Return the index whose value is the median among arr[a], arr[b], arr[c].
///
/// Uses only dyn-compatible logger methods (`cmp_ge` with swapped indices
/// instead of `cmp_le`, which requires `Self: Sized`).
fn median_index<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &[T],
    logger: &mut U,
    a: usize,
    b: usize,
    c: usize,
) -> usize {
    // cmp_ge(arr, x, y) = arr[x] >= arr[y], so cmp_ge(arr, b, a) ≡ arr[a] <= arr[b]
    let a_le_b = logger.cmp_ge(arr, b, a);
    let b_le_c = logger.cmp_ge(arr, c, b);

    if a_le_b {
        if b_le_c {
            b // a <= b <= c
        } else if logger.cmp_ge(arr, c, a) {
            c // a <= c < b
        } else {
            a // c < a <= b
        }
    } else if b_le_c {
        if logger.cmp_ge(arr, c, a) {
            a // b < a <= c
        } else {
            c // b <= c < a
        }
    } else {
        b // c <= b, b < a  →  median is b
    }
}
