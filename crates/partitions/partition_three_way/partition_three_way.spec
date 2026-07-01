# Component fragments OWNED by this crate (self-describing).

component pks_three_way
  type     ThreeWay
  label    three-way
  provides PartScheme
  uses     partition_three_way::ThreeWay
end

