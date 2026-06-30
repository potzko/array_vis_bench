# Component fragments OWNED by this crate (self-describing).

component mrot_grail
  type     GrailRotation
  label    grail
  provides Rotation
  uses     rotation_grail::GrailRotation
end

component rot_grail
  type     GrailRotation
  label    grail
  provides RotationKind
  category Rotation
  uses     rotation_grail::GrailRotation
end

