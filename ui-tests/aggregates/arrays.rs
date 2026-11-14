//@ compile-flags: -Zmir-opt-level=3

// CHECK-LABEL: // Pair: src_arg -> tgt_arg
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_index -> tgt_index
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_local -> tgt_local
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_nested -> tgt_nested
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_store -> tgt_store
// CHECK: // VERIFIED: tgt refines src.

#![crate_type = "lib"]

pub fn src_arg(xs: [i8; 3]) -> i8 {
    xs[1]
}

pub fn tgt_arg(xs: [i8; 3]) -> i8 {
    xs[1]
}

pub fn src_local(x: i8, y: i8, z: i8) -> [i8; 3] {
    let mut xs = [x, y, z];
    xs[1] = xs[0];
    xs
}

pub fn tgt_local(x: i8, _y: i8, z: i8) -> [i8; 3] {
    [x, x, z]
}

pub fn src_nested(xs: [[i8; 2]; 2]) -> i8 {
    xs[1][0]
}

pub fn tgt_nested(xs: [[i8; 2]; 2]) -> i8 {
    xs[1][0]
}

pub fn src_index(xs: [i8; 4], i: usize) -> i8 {
    xs[i]
}

pub fn tgt_index(xs: [i8; 4], i: usize) -> i8 {
    xs[i]
}

pub fn src_store(mut xs: [i8; 4], i: usize, v: i8) -> [i8; 4] {
    xs[i] = v;
    xs
}

pub fn tgt_store(mut xs: [i8; 4], i: usize, v: i8) -> [i8; 4] {
    xs[i] = v;
    xs
}
