# Component fragments OWNED by this crate (self-describing).

component mrot_drill
  type     DrillRotation
  label    drill
  provides Rotation
  uses     rotation_drill::DrillRotation
end

component rot_drill
  type     DrillRotation
  label    drill
  provides RotationKind
  category Rotation
  uses     rotation_drill::DrillRotation
end

