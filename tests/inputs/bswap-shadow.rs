#![crate_type="lib"]
#[inline(never)] fn bswap(x: u32) -> u32 { x }
pub fn src(x: u32) -> u32 { x.swap_bytes() }
pub fn tgt(x: u32) -> u32 { bswap(x) }
