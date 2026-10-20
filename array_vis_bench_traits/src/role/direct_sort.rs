//! `DirectSort`, the full sort role: sort a whole array in place. Every sort
//! driver type has an inherent `sort`, and `direct_sort!` is the one line impl
//! that makes it reachable through the role, so a `CorrectnessSuite` can be
//! generic over it.

use sort_logger::SortLogger;

pub trait DirectSort {
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U);
}

/// Implement [`DirectSort`] for a driver type by forwarding to its inherent
/// `sort`. After the type comes an optional `;` and the impl's generics, copied
/// from the impl that defines the inherent method:
///
/// ```text
/// direct_sort!(ShellSort<S>; S: GapSequence);
/// direct_sort!(BubbleSort);
/// ```
#[macro_export]
macro_rules! direct_sort {
    ($ty:ty $(; $($generics:tt)*)?) => {
        impl$(<$($generics)*>)? $crate::DirectSort for $ty {
            #[inline(always)]
            fn sort<T: Ord + Copy, U: ?Sized + $crate::sort_logger::SortLogger<T>>(
                arr: &mut [T],
                logger: &mut U,
            ) {
                <$ty>::sort(arr, logger)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::marker::PhantomData;

    // Sorts descending, so a call that reaches this inherent method is told
    // apart from the ascending `slice::sort`.
    fn descending<T: Ord + Copy>(arr: &mut [T]) {
        arr.sort_by(|a, b| b.cmp(a));
    }

    struct Plain;
    impl Plain {
        pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], _logger: &mut U) {
            descending(arr);
        }
    }
    direct_sort!(Plain);

    trait Marker {}
    impl Marker for () {}

    struct Generic<M: Marker, const N: usize>(PhantomData<M>);
    impl<M: Marker, const N: usize> Generic<M, N> {
        pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], _logger: &mut U) {
            descending(arr);
        }
    }
    direct_sort!(Generic<M, N>; M: Marker, const N: usize);

    #[test]
    fn the_role_reaches_the_inherent_sort_of_a_plain_type() {
        let mut arr = [1, 3, 2];
        <Plain as DirectSort>::sort(&mut arr, &mut sort_logger::NoOpLogger);
        assert_eq!(arr, [3, 2, 1]);
    }

    #[test]
    fn the_role_reaches_the_inherent_sort_of_a_generic_type() {
        let mut arr = [1usize, 3, 2];
        <Generic<(), 4> as DirectSort>::sort(&mut arr, &mut sort_logger::NoOpLogger);
        assert_eq!(arr, [3, 2, 1]);
    }
}
