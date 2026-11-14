//@ compile-flags: --edition=2024 -Coverflow-checks=false -Cdebug-assertions=false -Zmir-opt-level=3
//@ exit-code: 2
//@ allow-verifier-error
// CHECK: // Exit status: 2
// CHECK-LABEL: // Pair: src -> tgt
// CHECK: // ERROR
// CHECK-NEXT: // return type mismatch

#![crate_type="lib"]
pub fn src(x: u8) -> u8 { x }
pub fn tgt(x: u8) -> u16 { x as u16 }
