# Component fragments OWNED by this crate (self-describing).

component pks_moving
  type     MovingPivot
  label    moving pivot
  provides PartScheme
  uses     partition_moving_pivot::MovingPivot
end

