// CHECK-LABEL: // Pair: src_a -> tgt_a
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Return value mismatch:
// CHECK-LABEL: // Pair: src_b -> tgt_b
// CHECK: // VERIFIED: tgt refines src.

#![crate_type = "lib"]

#[repr(i8)]
pub enum Foo {
    A = -1,
    B = 2,
    C = -3,
}

pub fn src_a(i: Foo) -> i16 {
    match i {
        Foo::A => 255,
        Foo::B => 2,
        Foo::C => -3,
    }
}

pub fn tgt_a(i: Foo) -> i16 {
    i as i16
}

#[repr(i8)]
pub enum Bar {
    A = 1,
    B = 2,
    C = -3,
}

pub fn src_b(i: Bar) -> i16 {
    let i = i as i8;
    match i {
        -1 => 255,
        1 => 1,
        2 => 2,
        -3 => -3,
        _ => 0,
    }
}

pub fn tgt_b(i: Bar) -> i16 {
    i as i16
}
