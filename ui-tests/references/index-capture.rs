//@ compile-flags: --edition=2024 -Coverflow-checks=false -Cdebug-assertions=false -Zmir-opt-level=3
//@ exit-code: 1
// CHECK: // Exit status: 1
// CHECK-LABEL: // Pair: src -> tgt
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Return value mismatch:

// CHECK-NOT: // VERIFIED:

#![crate_type="lib"]
#![feature(core_intrinsics, custom_mir)]
#![allow(internal_features, unused_variables, unused_assignments)]
use core::intrinsics::mir::*;
#[custom_mir(dialect="runtime", phase="optimized")]
pub fn src(mut xs: [u8; 2]) -> [u8; 2] { mir! { let idx: usize; let p: &mut u8; { idx = 0; p = &mut xs[idx]; idx = 1; *p = 9; RET = xs; Return() } } }
pub fn tgt(mut xs: [u8; 2]) -> [u8; 2] { xs[1] = 9; xs }
