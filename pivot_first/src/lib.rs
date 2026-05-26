use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PivotQuality, PivotSelector,
};
use sort_logger::SortLogger;

pub struct FirstElement;

impl PivotSelector for FirstElement {
    const NAME: &'static str = "first";
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        _arr: &[T],
        _logger: &mut U,
    ) -> usize {
        0
    }
}

// Constant-time positional pivot — see the
// "Composable annotations" section in array_vis_bench for the rationale
// on `DEGENERATES = true` (worst case on sorted input).
impl HasTimeBounds for FirstElement {
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for FirstElement {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for FirstElement {
    const STABLE: bool = true;
}
impl PivotQuality for FirstElement {
    const DEGENERATES: bool = true;
}
