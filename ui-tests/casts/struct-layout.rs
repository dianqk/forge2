//@ compile-flags: --edition=2024 -Coverflow-checks=false -Cdebug-assertions=false
//@ exit-code: 1
// CHECK: // Exit status: 1
// CHECK-LABEL: // Pair: src -> tgt
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Return value mismatch:

// CHECK-LABEL: // Pair: src_control -> tgt_control
// CHECK: // VERIFIED: tgt refines src.

#![crate_type="lib"]
#[derive(Clone, Copy)] pub struct S { pub a: u8, pub b: u16, pub c: u8 }
pub fn src(x: S) -> u32 { unsafe { std::mem::transmute(x) } }
pub fn tgt(x: S) -> u32 { (x.a as u32) | ((x.b as u32) << 8) | ((x.c as u32) << 24) }

pub fn src_control(x: S) -> u32 { unsafe { std::mem::transmute(x) } }
pub fn tgt_control(x: S) -> u32 { (x.b as u32) | ((x.a as u32) << 16) | ((x.c as u32) << 24) }
