# Circle-sort family catalog fragment — OWNED by this crate, gathered by
# `spec_catalog` via `[package.metadata.array_vis_bench] spec = "circle.spec"`.
#
# Two slotted drivers — recursive (over an OPERATION ORDER) and bottom-up (over a
# size-traversal DIRECTION) — each also faceted by a FINISH strategy (run to
# convergence, or short-circuit at log2(n) and clean up with insertion), plus one
# zero-axis recursive-shaker base. Component names are PREFIXED `circle_` for
# global uniqueness in the merged catalog. `uses` paths resolve in the consumer
# (`spec_catalog`); they add no compile-time dep from this fragment.
#
# Counts: recursive 4 orders × 2 finishes = 8; bottom-up 4 directions × 2 finishes
# = 8; shaker 1. Total 17 — matches the 17 legacy `register_circle!` entries.

# ── Recursive driver (order × finish) ────────────────────────────────────────
component circle_recursive
  type     CircleRecursiveOf<{order},{finish}>
  label    circle sort<recursive: {order}, finish: {finish}>
  provides Sort
  menu     circle sorts / recursive
  slot     order  CircleOrder
  slot     finish CircleFinish
end

# ── Bottom-up driver (direction × finish) ────────────────────────────────────
component circle_bottom_up
  type     CircleBottomUpOf<{direction},{finish}>
  label    circle sort<bottom-up: {direction}, finish: {finish}>
  provides Sort
  menu     circle sorts / bottom-up
  slot     direction CircleDirection
  slot     finish    CircleFinish
end

# ── Recursive shaker (zero-axis) ─────────────────────────────────────────────
component circle_shaker_recursive
  type     CircleSortShakerRecursive
  label    circle sort (recursive shaker)
  provides Sort
  menu     circle sorts / recursive
end

# ── Recursive operation orders (CircleOrder) — labels match RecursiveOrder::NAME ─
component circle_pre_order
  type     PreOrder
  label    pre-order
  provides CircleOrder
end

component circle_left_mid_right
  type     LeftMidRight
  label    left-mid-right
  provides CircleOrder
end

component circle_right_mid_left
  type     RightMidLeft
  label    right-mid-left
  provides CircleOrder
end

component circle_post_order
  type     PostOrder
  label    post-order
  provides CircleOrder
end

# ── Bottom-up directions (CircleDirection) — labels match BottomUpDirection::NAME ─
component circle_decreasing
  type     Decreasing
  label    decreasing
  provides CircleDirection
end

component circle_increasing
  type     Increasing
  label    increasing
  provides CircleDirection
end

component circle_shaker_dec_inc
  type     ShakerDecInc
  label    shaker dec→inc
  provides CircleDirection
end

component circle_shaker_inc_dec
  type     ShakerIncDec
  label    shaker inc→dec
  provides CircleDirection
end

# ── Finish strategies (CircleFinish) ─────────────────────────────────────────
component circle_converge
  type     ConvergeFinish
  label    converge
  provides CircleFinish
end

component circle_sc_insertion
  type     InsertionShortCircuit
  label    short circuit: insertion
  provides CircleFinish
end
