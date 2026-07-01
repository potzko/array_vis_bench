# Component fragments OWNED by this crate (self-describing).

component fs_part_left_right
  type     LeftRightPartition
  label    left-right pointer
  provides CyclentPart
  uses     partition_hoare::LeftRightPartition
end

component qs_part_left_right
  type     LeftRightPartition
  label    left-right pointer
  provides QSPartition
  project  pivot QSPivotSingle
  uses     partition_hoare::LeftRightPartition
end

component pks_hoare
  type     LeftRightPartition
  label    left-right
  provides PartScheme
  uses     partition_hoare::LeftRightPartition
end

