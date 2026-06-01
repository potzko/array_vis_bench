use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PivotQuality, PivotSelector,
};
use sort_logger::SortLogger;

pub struct LastElement;

impl PivotSelector for LastElement {
    const NAME: &'static str = "last";
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        _logger: &mut U,
    ) -> usize {
        arr.len() - 1
    }
}

impl HasTimeBounds for LastElement {
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for LastElement {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for LastElement {
    const STABLE: bool = true;
}
impl PivotQuality for LastElement {
    const DEGENERATES: bool = true;
}
