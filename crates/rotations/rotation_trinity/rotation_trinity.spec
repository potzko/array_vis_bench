# Component fragments OWNED by this crate (self-describing).

component mrot_trinity
  type     TrinityRotation
  label    trinity
  provides Rotation
  uses     rotation_trinity::TrinityRotation
end

component rot_trinity
  type     TrinityRotation
  label    trinity
  provides RotationKind
  category Rotation
  uses     rotation_trinity::TrinityRotation
end

