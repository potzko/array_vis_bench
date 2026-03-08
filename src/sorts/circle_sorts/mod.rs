//! Circle sorts.
//!
//! # How circle sort works
//!
//! A single circle-sort *pass* over a range `[start, end]` compares the
//! outermost pair (`arr[start]` vs `arr[end]`), then the next pair inward,
//! and so on, swapping whenever the right element is smaller.  This "wraps
//! around" like a circle — hence the name.  The full sort repeats passes
//! until no swap occurs, guaranteeing convergence.
//!
//! # Two families, two abstractions
//!
//! ## Recursive — abstracted over *ordering* (`orderings.rs`)
//!
//! The recursive family also splits the range at the midpoint and sorts each
//! half.  At each recursion level there are three operations: `circle_pass`,
//! `recurse_left`, `recurse_right`.  The [`RecursiveOrder`] trait abstracts
//! over *which order* these three operations run.  Four orderings are
//! provided: `PreOrder`, `LeftMidRight`, `RightMidLeft`, and `PostOrder`.
//! A shaker variant alternates orderings with depth and is implemented
//! separately.
//!
//! ## Bottom-up — abstracted over *traversal direction* (`directions.rs`)
//!
//! The bottom-up family avoids recursion by iterating over all power-of-two
//! circle sizes explicitly.  The [`BottomUpDirection`] trait abstracts over
//! *in which order* those sizes are visited within one outer iteration.
//! Four directions are provided: `Decreasing`, `Increasing`, `ShakerDecInc`,
//! and `ShakerIncDec`.
//!
//! [`RecursiveOrder`]: orderings::RecursiveOrder
//! [`BottomUpDirection`]: directions::BottomUpDirection

pub mod circle_sort_bottom_up;
pub mod circle_sort_recursive;
pub mod circle_sort_shaker_recursive;
pub mod combinations;
pub mod directions;
pub mod finishing;
pub mod orderings;
pub mod sequences;

use crate::traits::log_traits::SortLogger;
use sequences::CIRCLE_ENTRIES;

pub fn fn_sort(
    arr: &mut [usize],
    logger: &mut dyn SortLogger<usize>,
    choice: &[String],
) -> Vec<String> {
    let name = choice.first().map(String::as_str).unwrap_or("");

    for entry in CIRCLE_ENTRIES {
        if entry.name == name {
            (entry.sort_vis)(arr, logger);
            return vec![format!("name: {}", name)];
        }
    }

    // Default: pre-order recursive
    use circle_sort_recursive::CircleSortRecursive;
    use orderings::PreOrder;
    CircleSortRecursive::<PreOrder>::sort(arr, logger);
    vec![format!("name: circle sort (recursive pre-order)")]
}

/// Returns the choice vec for `fn_sort` dispatch if `name` is a registered
/// circle-sort variant, otherwise `None`.
pub fn sort_choice(name: &str) -> Option<Vec<String>> {
    for entry in CIRCLE_ENTRIES {
        if entry.name == name {
            return Some(vec!["circle_sorts".to_string(), name.to_string()]);
        }
    }
    None
}
