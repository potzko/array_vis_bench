use crate::traits::log_traits::SortLogger;
use super::{Rotation, reverse};

/// Triple-reversal rotation (pre-1981).
pub struct ReversalRotation;

impl Rotation for ReversalRotation {
    const NAME: &'static str = "reversal";
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], split_ind: usize, logger: &mut U) {
        let n = arr.len();
        if split_ind == 0 || split_ind == n {
            return;
        }
        reverse(&mut arr[..split_ind], logger);
        reverse(&mut arr[split_ind..], logger);
        reverse(arr, logger);
    }
}

register_rotation!(ReversalRotation);
