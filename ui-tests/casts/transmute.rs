// CHECK-LABEL: // Pair: src_mismatch -> tgt_mismatch
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Return value mismatch:
// CHECK-LABEL: // Pair: src_pair_to_u32 -> tgt_pair_to_u32
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_to_bytes -> tgt_to_bytes
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_to_pair -> tgt_to_pair
// CHECK: // VERIFIED: tgt refines src.

#![crate_type = "lib"]

pub fn src_to_pair(x: u32) -> (u16, u16) {
    unsafe { std::mem::transmute(x) }
}

pub fn tgt_to_pair(x: u32) -> (u16, u16) {
    ((x & 0xffff) as u16, (x >> 16) as u16)
}

pub fn src_pair_to_u32(x: (u16, u16)) -> u32 {
    unsafe { std::mem::transmute(x) }
}

pub fn tgt_pair_to_u32(x: (u16, u16)) -> u32 {
    (x.0 as u32) | ((x.1 as u32) << 16)
}

pub fn src_to_bytes(x: u32) -> [u8; 4] {
    unsafe { std::mem::transmute(x) }
}

pub fn tgt_to_bytes(x: u32) -> [u8; 4] {
    [x as u8, (x >> 8) as u8, (x >> 16) as u8, (x >> 24) as u8]
}

pub fn src_mismatch(x: u32) -> (u16, u16) {
    unsafe { std::mem::transmute(x) }
}

pub fn tgt_mismatch(x: u32) -> (u16, u16) {
    ((x >> 16) as u16, (x & 0xffff) as u16)
}
