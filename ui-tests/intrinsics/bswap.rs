// CHECK-LABEL: // Pair: src_intrinsic -> tgt_intrinsic
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_method -> tgt_method
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_mismatch -> tgt_mismatch
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Return value mismatch:

#![feature(core_intrinsics)]
#![crate_type = "lib"]

pub fn src_method(x: u32) -> u32 {
    x.swap_bytes()
}

pub fn tgt_method(x: u32) -> u32 {
    x.swap_bytes()
}

pub fn src_intrinsic(x: u32) -> u32 {
    std::intrinsics::bswap(x)
}

pub fn tgt_intrinsic(x: u32) -> u32 {
    x.swap_bytes()
}

pub fn src_mismatch(x: u32) -> u32 {
    x.swap_bytes()
}

pub fn tgt_mismatch(x: u32) -> u32 {
    x
}
