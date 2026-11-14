// CHECK-LABEL: // Pair: src_a -> tgt_a
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Return value mismatch:
// CHECK-LABEL: // Pair: src_b -> tgt_b
// CHECK: // VERIFIED: tgt refines src.

#![crate_type = "lib"]

pub fn src_a(i: i8) -> i16 {
    i as i16
}

pub fn tgt_a(i: i8) -> i16 {
    !(i as i16)
}

pub fn src_b(i: u8) -> u8 {
    !i
}

pub fn tgt_b(i: u8) -> u8 {
    !i
}
