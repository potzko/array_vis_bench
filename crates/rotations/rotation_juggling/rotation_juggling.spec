# Component fragments OWNED by this crate (self-describing).

component mrot_juggling
  type     JugglingRotation
  label    juggling
  provides Rotation
  uses     rotation_juggling::JugglingRotation
end

component rot_juggling
  type     JugglingRotation
  label    juggling
  provides RotationKind
  category Rotation
  uses     rotation_juggling::JugglingRotation
end

