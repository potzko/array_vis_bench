# GapDistribution catalog fragment — OWNED by gap_distribution_lib.
#
# Spec-only (no query): contributes the 5 `GapDistribution` role components that
# `fun_sorts`'s `RandomShellSort<{dist}>` query fans its gap-distribution axis over.
# Mirrors the legacy `[[...components]]` metadata, in the spec-fragment shape.

component gd_uniform
  type     UniformDist
  label    uniform
  provides GapDistribution
  uses     gap_distribution_lib::UniformDist
end

component gd_parabolic
  type     ParabolicDist
  label    parabolic
  provides GapDistribution
  uses     gap_distribution_lib::ParabolicDist
end

component gd_cubic
  type     CubicDist
  label    cubic
  provides GapDistribution
  uses     gap_distribution_lib::CubicDist
end

component gd_log_uniform
  type     LogUniformDist
  label    log uniform
  provides GapDistribution
  uses     gap_distribution_lib::LogUniformDist
end

component gd_distinct_parabolic
  type     Distinct<ParabolicDist>
  label    distinct parabolic
  provides GapDistribution
  uses     gap_distribution_lib::Distinct gap_distribution_lib::ParabolicDist
end
