# Component fragments OWNED by this crate (self-describing).

component mrot_piston
  type     PistonRotation
  label    piston
  provides Rotation
  uses     rotation_piston::PistonRotation
end

component rot_piston
  type     PistonRotation
  label    piston
  provides RotationKind
  category Rotation
  uses     rotation_piston::PistonRotation
end

