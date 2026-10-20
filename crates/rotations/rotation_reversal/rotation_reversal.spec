# Component fragments OWNED by this crate (self-describing).

component rot_reversal
  type     ReversalRotation
  label    reversal
  provides Rotation
  category Rotation
  uses     rotation_reversal::ReversalRotation
end
