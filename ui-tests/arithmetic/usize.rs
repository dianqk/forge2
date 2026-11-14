//@ compile-flags: -Coverflow-checks=false

// CHECK-LABEL: // Pair: src -> tgt
// CHECK: // VERIFIED: tgt refines src.

#![crate_type = "lib"]

// FIXME: Supports 32 bit tests.

pub fn src(x: usize, y: usize) -> usize {
    x + y
}

pub fn tgt(x: usize, y: usize) -> usize {
    y + x
}
