# Component fragments OWNED by this crate (self-describing).

component mrot_contrev
  type     ContrevRotation
  label    contrev
  provides Rotation
  uses     rotation_contrev::ContrevRotation
end

component rot_contrev
  type     ContrevRotation
  label    contrev
  provides RotationKind
  category Rotation
  uses     rotation_contrev::ContrevRotation
end

