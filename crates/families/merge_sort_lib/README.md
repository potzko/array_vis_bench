# `merge_sort_lib`

Merge sort family: top down, bottom up, naive and natural classic merge sorts, the rotation merge sorts, the RotationMerge trait with its naive, smaller side and mask merges, `SmallerSideMerge` and `MaskMerge`, the `MergeStrategy` markers that leave the rotation to the caller, and the auxiliary merge the scratchspace backings plug into. The merge kinds the catalog emits as first class merges are declared here as well.

Its catalog components are declared in `merge.spec`.

Query `merge.avbs` builds: `TopDownMergeSorts`, `BottomUpMergeSorts`, `NaiveMergeSorts`, `NaturalMergeSorts`, `TopDownRotationMergeSorts`, `BottomUpRotationMergeSorts`.
