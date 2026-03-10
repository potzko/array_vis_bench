use crate::traits::log_traits::SortLogger;
pub fn merge_inplace<T: Ord + Copy, U: SortLogger<T>>(
    a: &[T],
    b: &[T],
    target: &mut [T],
    logger: &mut U,
) {
    let mut ind_a = 0;
    let mut ind_b = 0;
    let mut ind_c = 0;
    while ind_a < a.len() && ind_b < b.len() {
        if logger.cmp_le_accross(a, ind_a, b, ind_b) {
            logger.write_accross(a, ind_a, target, ind_c);
            ind_c += 1;
            ind_a += 1;
        } else {
            logger.write_accross(b, ind_b, target, ind_c);
            ind_c += 1;
            ind_b += 1;
        }
    }
    match a.len() - ind_a {
        0 => copy_from_slice(&b[ind_b..], &mut target[ind_c..], logger),
        _ => copy_from_slice(&a[ind_a..], &mut target[ind_c..], logger),
    };
}

pub fn copy_from_slice<T: Ord + Copy, U: SortLogger<T>>(a: &[T], b: &mut [T], logger: &mut U) {
    for i in 0..std::cmp::min(a.len(), b.len()) {
        logger.write_accross(a, i, b, i);
    }
}
