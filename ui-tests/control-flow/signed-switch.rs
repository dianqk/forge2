// CHECK-LABEL: // Pair: src -> tgt
// CHECK: // COUNTEREXAMPLE
// CHECK-NEXT: // Return value mismatch:

#![crate_type = "lib"]
#![feature(core_intrinsics, custom_mir)]
#![allow(internal_features, unused_assignments)]

use std::intrinsics::mir::*;

#[custom_mir(dialect = "built")]
pub fn src(i: i8) -> i16 {
    mir! {
        {
            match i {
                -1 => a1,
                2 => a2,
                -3 => a3,
                _ => unreachable,
            }
        }
        a1 = {
            RET = 255;
            Return()
        }
        a2 = {
            RET = 2;
            Return()
        }
        a3 = {
            RET = -3;
            Return()
        }
        unreachable = {
            Unreachable()
        }
    }
}

#[custom_mir(dialect = "built")]
pub fn tgt(i: i8) -> i16 {
    mir! {
        {
            match i {
                -1 => a1,
                2 => a2,
                -3 => a3,
                _ => unreachable,
            }
        }
        a1 = {
            RET = -1;
            Return()
        }
        a2 = {
            RET = 2;
            Return()
        }
        a3 = {
            RET = -3;
            Return()
        }
        unreachable = {
            Unreachable()
        }
    }
}
