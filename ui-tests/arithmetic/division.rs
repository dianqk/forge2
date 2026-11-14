//@ compile-flags: --edition=2024 -Coverflow-checks=false -Cdebug-assertions=false -Zmir-opt-level=3
//@ exit-code: 0
// CHECK: // Exit status: 0
// CHECK-LABEL: // Pair: src -> tgt
// CHECK: // VERIFIED: tgt refines src.

// CHECK-LABEL: // Pair: src_rem -> tgt_rem
// CHECK: // VERIFIED: tgt refines src.

#![crate_type="lib"]
pub fn src(x: i64, y: i64) -> i64 { x / y }
pub fn tgt(x: i64, y: i64) -> i64 { x / y }

pub fn src_rem(x: i64, y: i64) -> i64 { x % y }
pub fn tgt_rem(x: i64, y: i64) -> i64 { x % y }
