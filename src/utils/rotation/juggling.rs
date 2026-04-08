use crate::traits::log_traits::SortLogger;
use super::{Rotation, gcd};

/// Juggling rotation (1965): GCD cycle-based.
pub struct JugglingRotation;

impl Rotation for JugglingRotation {
    const NAME: &'static str = "juggling";
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], split_ind: usize, logger: &mut U) {
        let n = arr.len();
        let left = split_ind;
        if left == 0 || left == n {
            return;
        }
        let cycles = gcd(left, n);
        for start in 0..cycles {
            let saved = arr[start];
            let mut pta = start;
            loop {
                let ptb = pta + left;
                let ptb = if ptb >= n { ptb - n } else { ptb };
                if ptb == start {
                    break;
                }
                let v = arr[ptb];
                logger.write_data(arr, pta, v);
                pta = ptb;
            }
            logger.write_data(arr, pta, saved);
        }
    }
}

register_rotation!(JugglingRotation);
