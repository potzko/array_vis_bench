# AVL sort family catalog fragment, owned by avl_sort_lib and gathered by
# `spec_catalog` through `[package.metadata.array_vis_bench] spec = "avl.spec"`.
# One concrete sort, no slots. The cap is the largest node count whose three
# link fields fit the packed word, 2^19 - 1, see the constants in lib.rs.

component avl_sort
  type      AvlSort
  label     avl sort
  provides  Sort
  menu      avl sorts
  max_input 524287
end
