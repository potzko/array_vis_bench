# Index heap sort family catalog fragment, owned by index_heap_sort_lib and
# gathered by `spec_catalog` through `[package.metadata.array_vis_bench] spec = "index_heap.spec"`.
# Zero axis: a single concrete sort, no slots.

component index_heap_sort
  type      IndexHeapSort
  label     index heap sort
  provides  Sort
  menu      heap sorts / index
end
