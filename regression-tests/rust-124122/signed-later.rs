//@ forge-flags: --verify-pass MatchBranchSimplification_124122
//@ exit-code: 1
//@ compile-flags: --crate-type=lib

// CHECK: // Pass: MatchBranchSimplification_124122
// CHECK-LABEL: // Function: signed_later
// CHECK-COUNT-1: // COUNTEREXAMPLE
// CHECK-NEXT: // Return value mismatch:
// CHECK-NEXT: //   _1 (i) = Input::C (-3)
// CHECK-NEXT: //   before returns: 253
// CHECK-NEXT: //   after returns: -3
// CHECK: fn signed_later_before
// CHECK: switchInt
// CHECK: fn signed_later_after
// CHECK: _0 = cast _3 : i8 -> i16;
// CHECK-NOT: // COUNTEREXAMPLE

// The first two arms match signed widening; the third triggers the buggy fallback.
// https://github.com/rust-lang/rust/pull/124122

#[repr(i8)]
pub enum Input { A = -1, B = 2, C = -3 }

pub fn signed_later(i: Input) -> i16 {
    match i {
        Input::A => -1,
        Input::B => 2,
        Input::C => 253,
    }
}
