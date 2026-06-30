# Component fragments OWNED by this crate (self-describing).

component qs_nopivot
  type     NoPivot
  label    no pivot
  provides QSPivot QSPivotNone
  uses     array_vis_bench_traits::NoPivot
end

