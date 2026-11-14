//@ compile-flags: -Coverflow-checks=false

// CHECK-LABEL: // Pair: src_a -> tgt_a
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_b -> tgt_b
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_c -> tgt_c
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_d -> tgt_d
// CHECK: // VERIFIED: tgt refines src.
// CHECK-LABEL: // Pair: src_ref -> tgt_ref
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Reference parameter mutation mismatch:
// CHECK-LABEL: // Pair: src_ref_2 -> tgt_ref_2
// CHECK: // VERIFIED: tgt refines src.

#![crate_type = "lib"]

pub struct Pair {
    pub a: i8,
    pub b: i8,
}

pub fn src_a(mut a: i8, b: i8) -> i8 {
    a = a + b;
    a
}

pub fn tgt_a(a: i8, b: i8) -> i8 {
    a + b
}

pub fn src_b(mut a: i8, b: i8) -> i8 {
    let a = &mut a;
    *a = *a + b;
    *a
}

pub fn tgt_b(a: i8, b: i8) -> i8 {
    a + b
}

pub fn src_c(mut pair: Pair, b: i8) -> i8 {
    let a = &mut pair.a;
    *a = *a + b;
    pair.a
}

pub fn tgt_c(pair: Pair, b: i8) -> i8 {
    pair.a + b
}

pub fn src_d(x: &mut i8, b: i8) -> i8 {
    *x = *x + b;
    *x
}

pub fn tgt_d(x: &mut i8, b: i8) -> i8 {
    *x = b + *x;
    *x
}

pub fn src_ref(a: &mut i8, b: i8) -> i8 {
    *a = b;
    b
}

pub fn tgt_ref(a: &mut i8, b: i8) -> i8 {
    *a = b + 1;
    b
}

pub fn src_ref_2(a: &mut i8, b: &mut i8) -> i8 {
    *a = 1;
    *b = 2;
    *a + *b
}

pub fn tgt_ref_2(a: &mut i8, b: &mut i8) -> i8 {
    *a = 1;
    *b = 2;
    3
}
