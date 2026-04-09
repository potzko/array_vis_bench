use crate::sorts::merge_sorts::small_sort::{
    InsertionSmallSort, Network16SmallSort, NetworkSmallSort, NoSmallSort,
};
use super::partitions::{Block, Hoare, Lomuto, MovingPivot, ThreeWay};
use super::pivot_selectors::{
    FirstElement, LastElement, MedianOfMedians, MedianOfThree, MiddleElement, Ninther,
};
use super::quick_sort::QuickSort;
use super::dual_pivot_quick_sort::DualPivotQuickSort;

// ===========================================================================
// Classic (single-pivot) quicksort — partition × pivot × small-sort
// ===========================================================================

sort_registry_macro::sort_family! {
    type Sort = QuickSort<{P}, {V}, {SS}>;

    P {
        Lomuto      => "lomuto"
        Hoare       => "hoare"
        MovingPivot => "moving pivot"
        ThreeWay    => "three-way"
        Block       => "block"
    }

    V {
        FirstElement    => "first"
        MiddleElement   => "middle"
        LastElement     => "last"
        MedianOfThree   => "median of 3"
        MedianOfMedians => "median of medians"
        Ninther         => "ninther"
    }

    SS {
        NoSmallSort            => "no threshold"
        InsertionSmallSort<16> => "insertion: 16"
        InsertionSmallSort<32> => "insertion: 32"
        NetworkSmallSort       => "network: 8"
        Network16SmallSort     => "network: 16"
    }

    name   = "quick sort classic";
    big_o  = "O(N Log(N))";
    stable = false;
    direct_sort = true;
    path   = ["quick sorts", "classic", "{P}", "{V}", "{SS}"];
}

// ===========================================================================
// Dual-pivot quicksort — pivot1 × pivot2 × small-sort
// ===========================================================================

sort_registry_macro::sort_family! {
    type Sort = DualPivotQuickSort<{V1}, {V2}, {SS}>;

    V1 {
        FirstElement    => "first"
        MiddleElement   => "middle"
        LastElement     => "last"
        MedianOfThree   => "median of 3"
        MedianOfMedians => "median of medians"
        Ninther         => "ninther"
    }

    V2 {
        FirstElement    => "first"
        MiddleElement   => "middle"
        LastElement     => "last"
        MedianOfThree   => "median of 3"
        MedianOfMedians => "median of medians"
        Ninther         => "ninther"
    }

    SS {
        NoSmallSort            => "no threshold"
        InsertionSmallSort<16> => "insertion: 16"
        InsertionSmallSort<32> => "insertion: 32"
        NetworkSmallSort       => "network: 8"
        Network16SmallSort     => "network: 16"
    }

    name   = "quick sort dual pivot";
    big_o  = "O(N Log(N))";
    stable = false;
    direct_sort = true;
    path   = ["quick sorts", "dual pivot", "{V1}", "{V2}", "{SS}"];
}
