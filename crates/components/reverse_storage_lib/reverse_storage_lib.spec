# Component fragments OWNED by this crate (self-describing).

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

