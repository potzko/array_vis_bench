# Component fragments OWNED by this crate (self-describing).

component rot_auxiliary
  type     AuxiliaryRotation
  label    auxiliary
  provides Rotation
  category Rotation
  uses     rotation_auxiliary::AuxiliaryRotation
end
