// Exit status: 1

// Pair: src_intrinsic -> tgt_intrinsic
// NOTE: source FIR is identical to target FIR.
// VERIFIED: tgt refines src.

fn src_intrinsic(_1: u32) -> u32 {
  locals:
    _0: u32
    _1: u32
  blocks:
    bb0:
      _0 = bswap _1 : u32;
      goto bb1;
    bb1:
      return;
}

fn tgt_intrinsic(_1: u32) -> u32 {
  locals:
    _0: u32
    _1: u32
  blocks:
    bb0:
      _0 = bswap _1 : u32;
      goto bb1;
    bb1:
      return;
}

// Pair: src_method -> tgt_method
// NOTE: source FIR is identical to target FIR.
// VERIFIED: tgt refines src.

fn src_method(_1: u32) -> u32 {
  locals:
    _0: u32
    _1: u32
  blocks:
    bb0:
      _0 = bswap _1 : u32;
      goto bb1;
    bb1:
      return;
}

fn tgt_method(_1: u32) -> u32 {
  locals:
    _0: u32
    _1: u32
  blocks:
    bb0:
      _0 = bswap _1 : u32;
      goto bb1;
    bb1:
      return;
}

// Pair: src_mismatch -> tgt_mismatch
// COUNTEREXAMPLE
// Return value mismatch:
//   _1 (x) = 4294901760
//   src returns: 65535
//   tgt returns: 4294901760

fn src_mismatch(_1: u32) -> u32 {
  locals:
    _0: u32
    _1: u32
  blocks:
    bb0:
      _0 = bswap _1 : u32;
      goto bb1;
    bb1:
      return;
}

fn tgt_mismatch(_1: u32) -> u32 {
  locals:
    _0: u32
    _1: u32
  blocks:
    bb0:
      _0 = _1;
      return;
}

// Compiler diagnostics:
// warning: the feature `core_intrinsics` is internal to the compiler or standard library
//  --> ui-tests/intrinsics/bswap.rs:9:12
//   |
// 9 | #![feature(core_intrinsics)]
//   |            ^^^^^^^^^^^^^^^
//   |
//   = note: using it is strongly discouraged
//   = note: `#[warn(internal_features)]` on by default
//
// warning: 1 warning emitted
//
