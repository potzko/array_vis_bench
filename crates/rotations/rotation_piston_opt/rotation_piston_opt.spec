# Component fragments OWNED by this crate (self-describing).

component rot_piston_opt
  type     OptimizedPistonRotation
  label    optimized piston
  provides Rotation
end

# The 256 cell buffer finishes every range shorter than 771 cells, so below
# that the rotation above never runs a pass in place. This one always does.
component rot_piston_opt_in_place
  type     OptimizedPiston<2, 0>
  label    optimized piston<in place>
  provides Rotation
end

# A third window: five moves a column for three elements, against four for
# two, and the fastest over all splits from a thousand cells up.
component rot_piston_opt_three_windows
  type     OptimizedPiston<3, 256>
  label    optimized piston<3 windows>
  provides Rotation
end
