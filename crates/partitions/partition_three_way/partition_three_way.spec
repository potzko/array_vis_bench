# Component fragments OWNED by this crate (self-describing).

component fs_part_three_way
  type     ThreeWay
  label    three-way
  provides CyclentPart
  uses     partition_three_way::ThreeWay
end

component part_three_way
  type     ThreeWay
  label    three-way
  provides Partition
  project  pivot PivotSingle
  uses     partition_three_way::ThreeWay
end

component qs_part_three_way
  type     ThreeWay
  label    three-way
  provides QSPartition
  project  pivot QSPivotSingle
  uses     partition_three_way::ThreeWay
end

component pks_three_way
  type     ThreeWay
  label    three-way
  provides PartScheme
  uses     partition_three_way::ThreeWay
end

