//@ compile-flags: -Coverflow-checks=false

// CHECK-LABEL: // Pair: src -> tgt
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_rev -> tgt_rev
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Target reaches unreachable where source does not.
// CHECK-LABEL: // Pair: src_ub -> tgt_ub
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Source always reaches unreachable.

#![crate_type = "lib"]
#![feature(core_intrinsics)]
#![allow(internal_features, unused)]

use std::intrinsics::assume;

pub fn src(x: i32) -> i32 {
    unsafe {
        assume(x > 0);
    }
    x
}

pub fn tgt(x: i32) -> i32 {
    x
}

pub fn src_rev(x: i32) -> i32 {
    x
}

pub fn tgt_rev(x: i32) -> i32 {
    unsafe {
        assume(x > 0);
    }
    x
}

pub fn src_ub(x: i32) -> i32 {
    unsafe {
        assume(x > 0);
        assume(x < 0);
    }
    x
}

pub fn tgt_ub(x: i32) -> i32 {
    x + 1
}
