# Component fragments OWNED by this crate (self-describing).

component rot_grail
  type     GrailRotation
  label    grail
  provides Rotation
  category Rotation
  uses     rotation_grail::GrailRotation
end
