# Component fragments OWNED by this crate (self-describing).

component part_moving_pivot
  type     MovingPivot
  label    moving pivot
  provides Partition
  project  pivot PivotSingle
  uses     partition_moving_pivot::MovingPivot
end

component pks_moving
  type     MovingPivot
  label    moving pivot
  provides PartScheme
  uses     partition_moving_pivot::MovingPivot
end

