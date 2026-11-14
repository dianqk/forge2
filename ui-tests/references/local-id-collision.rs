//@ compile-flags: --edition=2024 -Coverflow-checks=false -Cdebug-assertions=false -Zmir-opt-level=3
//@ exit-code: 1
// CHECK: // Exit status: 1
// CHECK-LABEL: // Pair: src -> tgt
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Reference parameter mutation mismatch:

// CHECK-NOT: // VERIFIED:

#![crate_type="lib"]
#![feature(core_intrinsics, custom_mir)]
#![allow(internal_features, unused_variables, unused_assignments)]
use core::intrinsics::mir::*;
#[custom_mir(dialect="runtime", phase="optimized")]
pub fn src(p: &mut u8) -> u8 { mir! { { *p = 1; RET = 0; Return() } } }
#[custom_mir(dialect="runtime", phase="optimized")]
pub fn tgt(p: &mut u8) -> u8 { mir! { let temp: u8; { temp = 1; RET = 0; Return() } } }
