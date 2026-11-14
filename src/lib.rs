#![feature(rustc_private)]
#![feature(deref_patterns)]

extern crate rustc_abi;
extern crate rustc_const_eval;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;

mod analysis;
pub mod ir;
pub mod passes;
pub mod smt;
pub mod translate;
pub mod verify;

pub use analysis::{
    CollectedFunction, ForgeCallbacks, PassVerifyCallbacks, mir_enable_passes_after,
    mir_enable_passes_before,
};
