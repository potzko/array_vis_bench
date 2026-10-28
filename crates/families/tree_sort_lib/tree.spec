# Tree sort family catalog fragment, owned by tree_sort_lib and gathered by
# `spec_catalog` through `[package.metadata.array_vis_bench] spec = "tree.spec"`.
# One driver over the Tree axis, the rule that keeps the search tree in shape.
# The cap is the largest node count whose three link fields fit the packed
# word, 2^19 - 1, see the constants in src/nodes.rs.

component tree_sort
  type      TreeSort<{tree}>
  label     tree sort<{tree}>
  provides  Sort
  slot      tree Tree
  menu      tree sorts
  max_input 524287
end

component tree_avl
  type     Avl
  label    avl
  provides Tree
end

component tree_naive
  type     Naive
  label    naive
  provides Tree
end
