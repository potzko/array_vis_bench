# Component fragments OWNED by this crate (self-describing).

component mrot_gries_mills
  type     GriesMillsRotation
  label    gries-mills
  provides Rotation
  uses     rotation_gries_mills::GriesMillsRotation
end

component rot_gries_mills
  type     GriesMillsRotation
  label    gries-mills
  provides RotationKind
  category Rotation
  uses     rotation_gries_mills::GriesMillsRotation
end

