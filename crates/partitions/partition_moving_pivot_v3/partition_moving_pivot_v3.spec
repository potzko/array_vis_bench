# Component fragments OWNED by this crate (self-describing).

component part_moving_pivot_v3
  type     MovingPivotV3<ReversalRotation>
  label    moving pivot v3
  provides Partition
  project  pivot PivotSingle
  uses     partition_moving_pivot_v3::MovingPivotV3 rotation_reversal::ReversalRotation
end

