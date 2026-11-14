//@ compile-flags: -Coverflow-checks=false -Zmir-opt-level=3

// CHECK-LABEL: // Pair: src -> tgt
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Target reaches unreachable where source does not.
// CHECK-LABEL: // Pair: src_overflowing -> tgt_overflowing
// CHECK: // VERIFIED: tgt refines src.

#![crate_type = "lib"]
#![feature(core_intrinsics, custom_mir)]
#![allow(internal_features)]

pub fn src(a: i8, b: i8) -> i8 {
    a + b
}

pub unsafe fn tgt(a: i8, b: i8) -> i8 {
    std::intrinsics::unchecked_add(a, b)
}

pub fn src_overflowing(a: i8, b: i8) -> (i8, bool) {
    a.overflowing_add(b)
}

pub fn tgt_overflowing(a: i8, b: i8) -> (i8, bool) {
    b.overflowing_add(a)
}
