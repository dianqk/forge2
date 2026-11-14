//@ compile-flags: --edition=2024 -Coverflow-checks=false -Cdebug-assertions=false -Zmir-opt-level=3
//@ exit-code: 101
//@ output: diagnostics
// CHECK: // Exit status: 101
// CHECK: unsupported call terminator: callee=bswap

#![crate_type="lib"]
#[inline(never)] fn bswap(x: u32) -> u32 { x }
pub fn src(x: u32) -> u32 { x.swap_bytes() }
pub fn tgt(x: u32) -> u32 { bswap(x) }
