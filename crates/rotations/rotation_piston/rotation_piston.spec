# Component fragments OWNED by this crate (self-describing).

component rot_piston
  type     PistonRotation
  label    piston
  provides Rotation
  uses     rotation_piston::PistonRotation
end
