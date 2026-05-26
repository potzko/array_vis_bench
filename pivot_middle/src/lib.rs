use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PivotQuality, PivotSelector,
};
use sort_logger::SortLogger;

pub struct MiddleElement;

impl PivotSelector for MiddleElement {
    const NAME: &'static str = "middle";
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        _logger: &mut U,
    ) -> usize {
        arr.len() / 2
    }
}

impl HasTimeBounds for MiddleElement {
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for MiddleElement {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for MiddleElement {
    const STABLE: bool = true;
}
impl PivotQuality for MiddleElement {
    // Degenerate-safe in expectation for randomised input but *can*
    // still degenerate on adversarial input — conservatively `true`.
    const DEGENERATES: bool = true;
}
