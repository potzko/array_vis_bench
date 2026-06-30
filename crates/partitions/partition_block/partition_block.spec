# Component fragments OWNED by this crate (self-describing).

component fs_part_block
  type     Block
  label    block
  provides CyclentPart
  uses     partition_block::Block
end

component part_block
  type     Block
  label    block
  provides Partition
  project  pivot PivotSingle
  uses     partition_block::Block
end

component qs_part_block
  type     Block
  label    block
  provides QSPartition
  project  pivot QSPivotSingle
  uses     partition_block::Block
end

component pks_block
  type     Block
  label    block
  provides PartScheme
  uses     partition_block::Block
end

