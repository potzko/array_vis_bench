# Component fragments OWNED by this crate (self-describing).

component mrot_bridge
  type     BridgeRotation
  label    bridge
  provides Rotation
  uses     rotation_bridge::BridgeRotation
end

component rot_bridge
  type     BridgeRotation
  label    bridge
  provides RotationKind
  category Rotation
  uses     rotation_bridge::BridgeRotation
end

