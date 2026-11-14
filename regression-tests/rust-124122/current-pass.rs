//@ forge-flags: --verify-pass MatchBranchSimplification
//@ exit-code: 0

// CHECK: // Pass: MatchBranchSimplification
// CHECK-LABEL: // Function: signed_first
// CHECK: // VERIFIED: after refines before.
// CHECK-LABEL: // Function: signed_later
// CHECK: // VERIFIED: after refines before.

// Check the same two inputs against the pinned compiler's current, fixed pass.
#![crate_type = "lib"]
#[path = "signed-first.rs"]
pub mod first;
#[path = "signed-later.rs"]
pub mod later;
