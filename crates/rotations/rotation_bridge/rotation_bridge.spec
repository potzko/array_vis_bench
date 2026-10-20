# Component fragments OWNED by this crate (self-describing).

component rot_bridge
  type     BridgeRotation
  label    bridge
  provides Rotation
  uses     rotation_bridge::BridgeRotation
end
