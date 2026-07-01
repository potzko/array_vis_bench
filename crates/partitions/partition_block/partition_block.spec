# Component fragments OWNED by this crate (self-describing).

component pks_block
  type     Block
  label    block
  provides PartScheme
  uses     partition_block::Block
end

