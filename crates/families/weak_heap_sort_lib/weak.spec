# Weak-heap-sort family catalog fragment — OWNED by weak_heap_sort_lib.
# Gathered via [package.metadata.array_vis_bench] spec = "weak.spec".
#
# Slotted driver HeapSort<WeakHeapSort<{d},{r}>> over the shared HeapDir role
# (owned by heap.spec — referenced here, NOT redeclared) x this family's own
# ReverseStorage axis. 2 x 2 = 4 entries.

component weak_heap_sort
  type     HeapSort<WeakHeapSort<{d}, {r}>>
  label    weak heap sort<dir: {d}, storage: {r}>
  provides Sort
  category Sort
  menu     heap sorts / weak
  uses     heap_sort_lib::heap_sort::HeapSort weak_heap_sort_lib::weak_heap_sort::WeakHeapSort
  slot     d HeapDir       heap_dir_min_reverse
  slot     r ReverseStorage weak_byte_storage
end

# ── ReverseStorage axis (weak-owned) ─────────────────────────────────────────
component weak_byte_storage
  type     ByteStorage
  label    byte storage
  provides ReverseStorage
  uses     reverse_storage_lib::ByteStorage
end
component weak_bit_storage
  type     BitStorage
  label    bit storage
  provides ReverseStorage
  uses     reverse_storage_lib::BitStorage
end
