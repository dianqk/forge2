// CHECK-LABEL: // Pair: src_const_struct -> tgt_const_struct
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_const_struct_mismatch -> tgt_const_struct_mismatch
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Return value mismatch:
// CHECK-LABEL: // Pair: src_local -> tgt_local
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_nest -> tgt_nest
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Return value mismatch:
// CHECK-LABEL: // Pair: src_ret -> tgt_ret
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Return value mismatch:
// CHECK-LABEL: // Pair: src_struct -> tgt_struct
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_tuple -> tgt_tuple
// CHECK: // VERIFIED: tgt refines src.

#![crate_type = "lib"]

pub struct Pair {
    pub a: i32,
    pub b: i8,
}

pub struct NestPair {
    pub a: Pair,
    pub b: Pair,
}

pub fn src_struct(x: Pair) -> i32 {
    x.a
}

pub fn tgt_struct(x: Pair) -> i32 {
    x.a
}

pub fn src_tuple(x: (i32, i8)) -> i32 {
    x.0
}

pub fn tgt_tuple(x: (i32, i8)) -> i32 {
    x.0
}

pub fn src_const_struct() -> Pair {
    Pair { a: 7, b: -3 }
}

pub fn tgt_const_struct() -> Pair {
    Pair { a: 7, b: -3 }
}

pub fn src_const_struct_mismatch() -> Pair {
    Pair { a: 7, b: -3 }
}

pub fn tgt_const_struct_mismatch() -> Pair {
    Pair { a: 7, b: 0 }
}

pub fn src_local(x: Pair) -> i32 {
    let y = x;
    y.a
}

pub fn tgt_local(x: Pair) -> i32 {
    x.a
}

pub fn src_ret(x: i32, y: i8) -> (i32, i8) {
    (x, y)
}

pub fn tgt_ret(x: i32, _y: i8) -> (i32, i8) {
    (x, 0)
}

pub fn src_nest(x: NestPair) -> Pair {
    x.a
}

pub fn tgt_nest(x: NestPair) -> Pair {
    x.b
}
