//@ forge-flags: --verify-pass MatchBranchSimplification_124122
//@ exit-code: 0

// CHECK: // Pass: MatchBranchSimplification_124122
// CHECK-LABEL: // Function: boolean_match
// CHECK: // VERIFIED: after refines before.
// CHECK-LABEL: // Function: signed_widening
// CHECK: // VERIFIED: after refines before.
// CHECK-LABEL: // Function: unsigned_widening
// CHECK: // VERIFIED: after refines before.

#![crate_type = "lib"]

#[repr(i8)]
pub enum Signed { A = -1, B = 2, C = -3 }

pub fn signed_widening(i: Signed) -> i16 {
    match i {
        Signed::A => -1,
        Signed::B => 2,
        Signed::C => -3,
    }
}

#[repr(u8)]
pub enum Unsigned { A = 255, B = 2, C = 253 }

pub fn unsigned_widening(i: Unsigned) -> u16 {
    match i {
        Unsigned::A => 255,
        Unsigned::B => 2,
        Unsigned::C => 253,
    }
}

pub fn boolean_match(i: u8) -> bool {
    match i { 42 => true, _ => false }
}
