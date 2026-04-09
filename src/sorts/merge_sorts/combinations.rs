use crate::sorts::merge_sorts::top_down::TopDownMergeSort;
use crate::sorts::merge_sorts::bottom_up::BottomUpMergeSort;
use crate::sorts::merge_sorts::top_down_mirror::TopDownMirrorMergeSort;
use crate::sorts::merge_sorts::naive::NaiveMergeSort;
use crate::sorts::merge_sorts::natural::NaturalMergeSort;
use crate::sorts::merge_sorts::timsort::TimSort;
use crate::sorts::merge_sorts::small_sort::{NoSmallSort, InsertionSmallSort, NetworkSmallSort, Network16SmallSort};
use crate::sorts::merge_sorts::rotation::TopDownRotationMergeSort;
use crate::sorts::merge_sorts::rotation::BottomUpRotationMergeSort;
use crate::sorts::merge_sorts::rotation_merge::{NaiveRotationMerge, SmallerSideRotationMerge};
use crate::utils::rotation::{
    ReversalRotation, AuxiliaryRotation, BridgeRotation, ContrevRotation,
    TrinityRotation, GriesMillsRotation, GrailRotation, PistonRotation,
    HelixRotation, DrillRotation, JugglingRotation,
};

// ===========================================================================
// Classic merge sorts
// ===========================================================================

sort_registry_macro::sort_family! {
    type Sort = TopDownMergeSort<{SS}, {PP}, {EE}>;

    SS {
        NoSmallSort            => "no threshold"
        InsertionSmallSort<16> => "insertion: 16"
        InsertionSmallSort<32> => "insertion: 32"
        NetworkSmallSort       => "network: 8"
        Network16SmallSort     => "network: 16"
    }
    PP {
        false => ""
        true  => "ping-pong"
    }
    EE {
        false => ""
        true  => "early-exit"
    }

    name        = "merge sort";
    big_o       = "O(N log N)";
    stable      = true;
    direct_sort = true;
    path        = ["merge sorts", "classic", "top-down", "{variant}"];
}

sort_registry_macro::sort_family! {
    type Sort = BottomUpMergeSort<{SS}, {PP}, {EE}>;

    SS {
        NoSmallSort            => "no threshold"
        InsertionSmallSort<16> => "insertion: 16"
        InsertionSmallSort<32> => "insertion: 32"
        NetworkSmallSort       => "network: 8"
        Network16SmallSort     => "network: 16"
    }
    PP {
        false => ""
        true  => "ping-pong"
    }
    EE {
        false => ""
        true  => "early-exit"
    }

    name        = "bottom-up merge sort";
    big_o       = "O(N log N)";
    stable      = true;
    direct_sort = true;
    path        = ["merge sorts", "classic", "bottom-up", "{variant}"];
}

sort_registry_macro::sort_family! {
    type Sort = TopDownMirrorMergeSort<{SS}, {PP}, {EE}>;

    SS {
        NoSmallSort            => "no threshold"
        InsertionSmallSort<16> => "insertion: 16"
        InsertionSmallSort<32> => "insertion: 32"
        NetworkSmallSort       => "network: 8"
        Network16SmallSort     => "network: 16"
    }
    PP {
        false => ""
        true  => "ping-pong"
    }
    EE {
        false => ""
        true  => "early-exit"
    }

    name        = "top-down mirror merge sort";
    big_o       = "O(N log N)";
    stable      = true;
    direct_sort = true;
    path        = ["merge sorts", "classic", "top-down mirror", "{variant}"];
}

sort_registry_macro::sort_family! {
    type Sort = NaiveMergeSort<{SS}>;

    SS {
        NoSmallSort            => "no threshold"
        InsertionSmallSort<16> => "insertion: 16"
        InsertionSmallSort<32> => "insertion: 32"
        NetworkSmallSort       => "network: 8"
        Network16SmallSort     => "network: 16"
    }

    name        = "naive merge sort";
    big_o       = "O(N log N)";
    stable      = true;
    direct_sort = true;
    path        = ["merge sorts", "classic", "naive", "{variant}"];
}

sort_registry_macro::sort_family! {
    type Sort = NaturalMergeSort<{PP}, {EE}>;

    PP {
        false => ""
        true  => "ping-pong"
    }
    EE {
        false => ""
        true  => "early-exit"
    }

    name        = "natural merge sort";
    big_o       = "O(N log N)";
    stable      = true;
    direct_sort = true;
    path        = ["merge sorts", "classic", "natural", "{variant}"];
}

sort_registry_macro::sort_family! {
    type Sort = TimSort<{Gallop}>;

    Gallop {
        false => ""
        true  => "gallop"
    }

    name        = "timsort";
    big_o       = "O(N log N)";
    stable      = true;
    direct_sort = true;
    path        = ["merge sorts", "miscellaneous", "timsort", "{variant}"];
}

// ===========================================================================
// Rotation merge sorts
//
// 4 sort_family! calls: {top-down, bottom-up} × {naive, smaller-side} merge,
// each with axes for the rotation algorithm (R) and small-sort strategy (SS).
// ===========================================================================

sort_registry_macro::sort_family! {
    type Sort = TopDownRotationMergeSort<{SS}, NaiveRotationMerge<{R}>, false>;

    R {
        ReversalRotation   => "reversal"
        AuxiliaryRotation  => "auxiliary"
        BridgeRotation     => "bridge"
        ContrevRotation    => "contrev"
        TrinityRotation    => "trinity"
        GriesMillsRotation => "gries-mills"
        GrailRotation      => "grail"
        PistonRotation     => "piston"
        HelixRotation      => "helix"
        DrillRotation      => "drill"
        JugglingRotation   => "juggling"
    }
    SS {
        NoSmallSort            => "no threshold"
        InsertionSmallSort<16> => "insertion: 16"
        InsertionSmallSort<32> => "insertion: 32"
        NetworkSmallSort       => "network: 8"
        Network16SmallSort     => "network: 16"
    }

    name        = "rotation merge sort";
    big_o       = "O(N log N)";
    stable      = true;
    direct_sort = true;
    path        = ["merge sorts", "rotation", "top-down", "{R}", "{SS}"];
}

sort_registry_macro::sort_family! {
    type Sort = TopDownRotationMergeSort<{SS}, SmallerSideRotationMerge<{R}>, false>;

    R {
        ReversalRotation   => "reversal"
        AuxiliaryRotation  => "auxiliary"
        BridgeRotation     => "bridge"
        ContrevRotation    => "contrev"
        TrinityRotation    => "trinity"
        GriesMillsRotation => "gries-mills"
        GrailRotation      => "grail"
        PistonRotation     => "piston"
        HelixRotation      => "helix"
        DrillRotation      => "drill"
        JugglingRotation   => "juggling"
    }
    SS {
        NoSmallSort            => "no threshold"
        InsertionSmallSort<16> => "insertion: 16"
        InsertionSmallSort<32> => "insertion: 32"
        NetworkSmallSort       => "network: 8"
        Network16SmallSort     => "network: 16"
    }

    name        = "rotation merge sort<smaller-side>";
    big_o       = "O(N log N)";
    stable      = true;
    direct_sort = true;
    path        = ["merge sorts", "rotation", "top-down smaller-side", "{R}", "{SS}"];
}

sort_registry_macro::sort_family! {
    type Sort = BottomUpRotationMergeSort<{SS}, NaiveRotationMerge<{R}>, false>;

    R {
        ReversalRotation   => "reversal"
        AuxiliaryRotation  => "auxiliary"
        BridgeRotation     => "bridge"
        ContrevRotation    => "contrev"
        TrinityRotation    => "trinity"
        GriesMillsRotation => "gries-mills"
        GrailRotation      => "grail"
        PistonRotation     => "piston"
        HelixRotation      => "helix"
        DrillRotation      => "drill"
        JugglingRotation   => "juggling"
    }
    SS {
        NoSmallSort            => "no threshold"
        InsertionSmallSort<16> => "insertion: 16"
        InsertionSmallSort<32> => "insertion: 32"
        NetworkSmallSort       => "network: 8"
        Network16SmallSort     => "network: 16"
    }

    name        = "bottom-up rotation merge sort";
    big_o       = "O(N log N)";
    stable      = true;
    direct_sort = true;
    path        = ["merge sorts", "rotation", "bottom-up", "{R}", "{SS}"];
}

sort_registry_macro::sort_family! {
    type Sort = BottomUpRotationMergeSort<{SS}, SmallerSideRotationMerge<{R}>, false>;

    R {
        ReversalRotation   => "reversal"
        AuxiliaryRotation  => "auxiliary"
        BridgeRotation     => "bridge"
        ContrevRotation    => "contrev"
        TrinityRotation    => "trinity"
        GriesMillsRotation => "gries-mills"
        GrailRotation      => "grail"
        PistonRotation     => "piston"
        HelixRotation      => "helix"
        DrillRotation      => "drill"
        JugglingRotation   => "juggling"
    }
    SS {
        NoSmallSort            => "no threshold"
        InsertionSmallSort<16> => "insertion: 16"
        InsertionSmallSort<32> => "insertion: 32"
        NetworkSmallSort       => "network: 8"
        Network16SmallSort     => "network: 16"
    }

    name        = "bottom-up rotation merge sort<smaller-side>";
    big_o       = "O(N log N)";
    stable      = true;
    direct_sort = true;
    path        = ["merge sorts", "rotation", "bottom-up smaller-side", "{R}", "{SS}"];
}
