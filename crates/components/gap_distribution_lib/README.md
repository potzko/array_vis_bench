# `gap_distribution_lib`

Gap distributions for random shell sort: uniform, parabolic, cubic, log uniform and distinct parabolic. The trait lives here because it needs rand.

Catalog fragment `gap_distribution.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `gd_uniform` | `UniformDist` | `uniform` | GapDistribution |  |
| `gd_parabolic` | `ParabolicDist` | `parabolic` | GapDistribution |  |
| `gd_cubic` | `CubicDist` | `cubic` | GapDistribution |  |
| `gd_log_uniform` | `LogUniformDist` | `log uniform` | GapDistribution |  |
| `gd_distinct_parabolic` | `Distinct<ParabolicDist>` | `distinct parabolic` | GapDistribution |  |
