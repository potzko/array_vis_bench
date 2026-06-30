# Component fragments OWNED by this crate (self-describing).

component mrot_reversal
  type     ReversalRotation
  label    reversal
  provides Rotation
  uses     rotation_reversal::ReversalRotation
end

component rot_reversal
  type     ReversalRotation
  label    reversal
  provides RotationKind
  category Rotation
  uses     rotation_reversal::ReversalRotation
end

