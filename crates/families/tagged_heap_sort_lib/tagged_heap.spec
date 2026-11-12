# Tagged heap sort family catalog fragment, owned by tagged_heap_sort_lib and
# gathered by `spec_catalog` through `[package.metadata.array_vis_bench] spec =
# "tagged_heap.spec"`. One driver and no axis: the stable heap sort whose extra
# data is a stamp per element a pop has moved.

component tagged_heap_sort
  type      TaggedHeapSort
  label     tagged heap sort
  provides  Sort
  menu      heap sorts / tagged
end
