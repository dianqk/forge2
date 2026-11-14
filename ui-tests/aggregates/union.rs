//@ compile-flags: --edition=2024 -Coverflow-checks=false -Cdebug-assertions=false
//@ exit-code: 1
// CHECK: // Exit status: 1
// CHECK-LABEL: // Pair: src -> tgt
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Return value mismatch:

// CHECK-LABEL: // Pair: src_construct -> tgt_construct
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_control -> tgt_control
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_signed -> tgt_signed
// CHECK: // VERIFIED: tgt refines src.

#![crate_type="lib"]
#[repr(C)]
pub union U { pub a: u8, pub b: u8 }
pub fn src(mut u: U) -> u8 { u.a = 9; unsafe { u.b } }
pub fn tgt(u: U) -> u8 { unsafe { u.b } }

pub fn src_construct(x: u8) -> u8 { let u = U { a: x }; unsafe { u.b } }

pub fn tgt_construct(x: u8) -> u8 { x }

pub fn src_control(mut u: U) -> u8 { u.a = 9; unsafe { u.b } }

pub fn tgt_control(_u: U) -> u8 { 9 }

#[repr(C)]
pub union Signed { pub unsigned: u32, pub signed: i32 }
pub fn src_signed(x: u32) -> i32 { let mut u = Signed { unsigned: 0 }; u.unsigned = x; unsafe { u.signed } }
pub fn tgt_signed(x: u32) -> i32 { x as i32 }
