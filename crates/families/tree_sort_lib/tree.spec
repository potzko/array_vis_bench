# Tree sort family catalog fragment, owned by tree_sort_lib and gathered by
# `spec_catalog` through `[package.metadata.array_vis_bench] spec = "tree.spec"`.
# One driver over the Tree axis, the rule that keeps the search tree in shape.
# The AVL tree is declared once per slack, since the spec has no integer enumeration,
# and `FixedAvl` is the classic one written for slack 1 alone.

component tree_sort
  type      TreeSort<{tree}>
  label     tree sort<{tree}>
  provides  Sort
  slot      tree Tree
  menu      tree sorts
end

component tree_avl1
  type     Avl<1>
  label    avl: 1
  provides Tree
end

component tree_avl2
  type     Avl<2>
  label    avl: 2
  provides Tree
end

component tree_avl4
  type     Avl<4>
  label    avl: 4
  provides Tree
end

component tree_avl8
  type     Avl<8>
  label    avl: 8
  provides Tree
end

component tree_avl16
  type     Avl<16>
  label    avl: 16
  provides Tree
end

component tree_avl32
  type     Avl<32>
  label    avl: 32
  provides Tree
end

component tree_fixed_avl
  type     FixedAvl
  label    avl: fixed 1
  provides Tree
end

component tree_naive
  type     Naive
  label    naive
  provides Tree
end
