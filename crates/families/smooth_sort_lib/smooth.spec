# Smooth sort family catalog fragment, owned by smooth_sort_lib and gathered by
# `spec_catalog` through `[package.metadata.array_vis_bench] spec =
# "smooth.spec"`. One driver over the shared HeapDir role (declared in
# heap.spec, referenced here and not declared again) and three axes of its own:
# the shape of the trees the heap is a forest of, the sift of one tree and the
# build of the forest.

component smooth_shape_leonardo
  type     Leonardo
  label    leonardo
  provides SmoothShape
end
component smooth_shape_perfect
  type     Perfect
  label    perfect
  provides SmoothShape
end

component smooth_sift_pairs
  type     Pairs
  label    pairs
  provides SmoothSift
end
component smooth_sift_bottom_up
  type     BottomUp
  label    bottom up
  provides SmoothSift
end

component smooth_build_insert
  type     Insert
  label    insert
  provides SmoothBuild
end
component smooth_build_lazy_insert
  type     LazyInsert
  label    lazy insert
  provides SmoothBuild
end
component smooth_build_heapify
  type     Heapify
  label    heapify
  provides SmoothBuild
end

component smooth_sort
  type     SmoothSort<{d}, {s}, {f}, {b}>
  label    smooth sort<dir: {d}, trees: {s}, sift: {f}, build: {b}>
  provides Sort
  menu     heap sorts / smooth
  slot     d HeapDir
  slot     s SmoothShape
  slot     f SmoothSift
  slot     b SmoothBuild
end
