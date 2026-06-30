# Component fragments OWNED by this crate (self-describing).

component mrot_auxiliary
  type     AuxiliaryRotation
  label    auxiliary
  provides Rotation
  uses     rotation_auxiliary::AuxiliaryRotation
end

component rot_auxiliary
  type     AuxiliaryRotation
  label    auxiliary
  provides RotationKind
  category Rotation
  uses     rotation_auxiliary::AuxiliaryRotation
end

