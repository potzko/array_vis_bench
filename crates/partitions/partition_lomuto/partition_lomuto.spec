# Component fragments OWNED by this crate (self-describing).

component fs_part_left_left
  type     LeftLeftPartition
  label    left-left pointer
  provides CyclentPart
  uses     partition_lomuto::LeftLeftPartition
end

component part_left_left
  type     LeftLeftPartition
  label    left-left
  provides Partition
  project  pivot PivotSingle
  uses     partition_lomuto::LeftLeftPartition
end

component qs_part_left_left
  type     LeftLeftPartition
  label    left-left pointer
  provides QSPartition
  project  pivot QSPivotSingle
  uses     partition_lomuto::LeftLeftPartition
end

component pks_lomuto
  type     LeftLeftPartition
  label    left-left
  provides PartScheme
  uses     partition_lomuto::LeftLeftPartition
end

