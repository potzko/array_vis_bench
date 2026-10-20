# Component fragments OWNED by scratchspace_lib (self-describing), gathered by
# spec_catalog via `[package.metadata.array_vis_bench] spec = "scratchspace.spec"`.
#
# `Auxiliary` is the scratchspace/backing role — how much working buffer a merge
# is given. `AuxiliaryMerge<{aux}>` is a RotationMerge that consumes one; declared
# as a Category::Merge kind so it visualises / benches / tests through the existing
# merge pipeline. spec_catalog's `Merges: MergeAlg<_>` query enumerates every
# MergeKind, so `mk_aux` joins automatically (× 3 Auxiliary backings).

component aux_full
  type     FullBuffer
  label    full
  provides Auxiliary
  uses     scratchspace_lib::FullBuffer
end

component aux_half
  type     HalfBuffer
  label    half
  provides Auxiliary
  uses     scratchspace_lib::HalfBuffer
end

component aux_none
  type     NoBuffer
  label    none
  provides Auxiliary
  uses     scratchspace_lib::NoBuffer
end

component mk_aux
  type     AuxiliaryMerge<{aux}>
  label    auxiliary<{aux}>
  provides MergeKind
  uses     scratchspace_lib::AuxiliaryMerge
  slot     aux Auxiliary
end

# ── Scratchspace backings — provide the `Scratchspace` role (the aux strategy);
# a slot filled by workflows and the nightmare sort. Not emitted on their own.

component bk_full
  type     FullScratch
  label    full
  provides Scratchspace
  uses     scratchspace_lib::FullScratch
end

component bk_sqrt
  type     SqrtScratch
  label    sqrt
  provides Scratchspace
  uses     scratchspace_lib::SqrtScratch
end

component bk_none
  type     NoScratch
  label    none
  provides Scratchspace
  uses     scratchspace_lib::NoScratch
end

# ── Workflows — each a top-level Category::Scratchspace kind that drives a
# distinct routine over the ops, slotting a backing. Enumerated (× 3 backings) by
# spec_catalog's `Scratchspaces: ScratchspaceAlg<_>`.

component wf_exercise
  type     Exercise<{backing}>
  label    exercise<{backing}>
  provides ScratchspaceKind
  uses     scratchspace_lib::Exercise
  slot     backing Scratchspace
end

component wf_park_sweep
  type     ParkSweep<{backing}>
  label    park-sweep<{backing}>
  provides ScratchspaceKind
  uses     scratchspace_lib::ParkSweep
  slot     backing Scratchspace
end

component wf_park_loop
  type     ParkLoop<{backing}>
  label    park-loop<{backing}>
  provides ScratchspaceKind
  uses     scratchspace_lib::ParkLoop
  slot     backing Scratchspace
end

component wf_relocate
  type     Relocations<{backing}>
  label    relocate<{backing}>
  provides ScratchspaceKind
  uses     scratchspace_lib::Relocations
  slot     backing Scratchspace
end

# ── The nightmare: a scratchspace-backed natural merge SORT (Category::Sort),
# slotting a backing. Queried by scratchspace.avbs.

component ss_nightmare
  type     ScratchspaceMergeSort<{backing}>
  label    scratchspace-nightmare<{backing}>
  provides Sort
  adaptive true
  menu     fun / scratchspace-nightmare
  uses     scratchspace_lib::ScratchspaceMergeSort
  slot     backing Scratchspace
end
