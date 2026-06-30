# Component fragments OWNED by this crate (self-describing).

component mrot_helix
  type     HelixRotation
  label    helix
  provides Rotation
  uses     rotation_helix::HelixRotation
end

component rot_helix
  type     HelixRotation
  label    helix
  provides RotationKind
  category Rotation
  uses     rotation_helix::HelixRotation
end

