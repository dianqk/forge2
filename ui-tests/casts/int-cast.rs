// CHECK-LABEL: // Pair: src -> tgt
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Return value mismatch:

#![crate_type = "lib"]

pub fn src(i: i8) -> i16 {
    match i {
        -1 => 255,
        2 => 2,
        -3 => -3,
        _ => i as i16,
    }
}

pub fn tgt(i: i8) -> i16 {
    i as i16
}
