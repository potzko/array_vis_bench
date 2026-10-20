# Component fragments OWNED by this crate (self-describing).

component rot_bridge
  type     BridgeRotation
  label    bridge
  provides Rotation
  category Rotation
  uses     rotation_bridge::BridgeRotation
end
