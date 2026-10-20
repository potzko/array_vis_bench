# Component fragments OWNED by this crate (self-describing).

component rot_drill
  type     DrillRotation
  label    drill
  provides Rotation
  category Rotation
  uses     rotation_drill::DrillRotation
end
