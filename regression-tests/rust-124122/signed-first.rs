//@ forge-flags: --verify-pass MatchBranchSimplification_124122
//@ exit-code: 1
//@ compile-flags: --crate-type=lib

// CHECK: // Pass: MatchBranchSimplification_124122
// CHECK-LABEL: // Function: signed_first
// CHECK-COUNT-1: // COUNTEREXAMPLE
// CHECK-NEXT: // Return value mismatch:
// CHECK-NEXT: //   _1 (i) = Input::A (-1)
// CHECK-NEXT: //   before returns: 255
// CHECK-NEXT: //   after returns: -1
// CHECK: fn signed_first_before
// CHECK: switchInt
// CHECK: fn signed_first_after
// CHECK: _0 = cast _3 : i8 -> i16;
// CHECK-NOT: // COUNTEREXAMPLE

// First two arms exercise the erroneous unsigned fallback in can_simplify.
// https://github.com/rust-lang/rust/pull/124122

#[repr(i8)]
pub enum Input { A = -1, B = 2, C = -3 }

pub fn signed_first(i: Input) -> i16 {
    match i {
        Input::A => 255,
        Input::B => 2,
        Input::C => -3,
    }
}
