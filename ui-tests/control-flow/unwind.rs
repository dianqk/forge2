//@ compile-flags: -Zmir-opt-level=3

// CHECK-LABEL: // Pair: src_source_unwind -> tgt_source_unwind
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Source unwinds where target does not.
// CHECK-LABEL: // Pair: src_target_unwind -> tgt_target_unwind
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Target unwinds where source does not.
// CHECK-LABEL: // Pair: src_unwind_reason -> tgt_unwind_reason
// CHECK: // VERIFIED: tgt refines src.

#![crate_type = "lib"]

pub fn src_target_unwind(xs: [i8; 4], i: usize) -> i8 {
    xs[i & 3]
}

pub fn tgt_target_unwind(xs: [i8; 4], i: usize) -> i8 {
    xs[i]
}

pub fn src_source_unwind(xs: [i8; 4], i: usize) -> i8 {
    xs[i]
}

pub fn tgt_source_unwind(xs: [i8; 4], i: usize) -> i8 {
    xs[i & 3]
}

pub fn src_unwind_reason(xs: [i8; 4], i: usize) -> i8 {
    // FIXME: Forge currently matches only the unwind outcome, not the unwind reason.
    xs[i + 4]
}

pub fn tgt_unwind_reason(xs: [i8; 4], i: usize) -> i8 {
    xs[i + 5]
}
