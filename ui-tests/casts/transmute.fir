// Exit status: 1

// Pair: src_mismatch -> tgt_mismatch
// COUNTEREXAMPLE
// Return value mismatch:
//   _1 (x) = 65535
//   src returns: (65535, 0)
//   tgt returns: (0, 65535)

fn src_mismatch(_1: u32) -> (u16, u16) {
  locals:
    _0: (u16, u16)
    _1: u32
  blocks:
    bb0:
      _0 = transmute _1 : u32 -> (u16, u16);
      return;
}

fn tgt_mismatch(_1: u32) -> (u16, u16) {
  locals:
    _0: (u16, u16)
    _1: u32
    _2: u16
    _3: u32
    _4: u32
    _5: bool
    _6: u16
    _7: u32
  blocks:
    bb0:
      _4 = cast 16:i32 : i32 -> u32;
      _5 = _4 < 32:u32 : u32;
      assert _5 then bb1 else unwind;
    bb1:
      _3 = _1 >> 16:i32 : u32;
      _2 = cast _3 : u32 -> u16;
      _7 = _1 & 65535:u32 : u32;
      _6 = cast _7 : u32 -> u16;
      _0 = (_2, _6);
      return;
}

// Pair: src_pair_to_u32 -> tgt_pair_to_u32
// VERIFIED: tgt refines src.

fn src_pair_to_u32(_1: (u16, u16)) -> u32 {
  locals:
    _0: u32
    _1: (u16, u16)
  blocks:
    bb0:
      _0 = transmute _1 : (u16, u16) -> u32;
      return;
}

fn tgt_pair_to_u32(_1: (u16, u16)) -> u32 {
  locals:
    _0: u32
    _1: (u16, u16)
    _2: u32
    _3: u16
    _4: u32
    _5: u32
    _6: u16
    _7: u32
    _8: bool
  blocks:
    bb0:
      _3 = _1.0;
      _2 = cast _3 : u16 -> u32;
      _6 = _1.1;
      _5 = cast _6 : u16 -> u32;
      _7 = cast 16:i32 : i32 -> u32;
      _8 = _7 < 32:u32 : u32;
      assert _8 then bb1 else unwind;
    bb1:
      _4 = _5 << 16:i32 : u32;
      _0 = _2 | _4 : u32;
      return;
}

// Pair: src_to_bytes -> tgt_to_bytes
// VERIFIED: tgt refines src.

fn src_to_bytes(_1: u32) -> (u8, u8, u8, u8) {
  locals:
    _0: (u8, u8, u8, u8)
    _1: u32
  blocks:
    bb0:
      _0 = transmute _1 : u32 -> (u8, u8, u8, u8);
      return;
}

fn tgt_to_bytes(_1: u32) -> (u8, u8, u8, u8) {
  locals:
    _0: (u8, u8, u8, u8)
    _1: u32
    _2: u8
    _3: u8
    _4: u32
    _5: u32
    _6: bool
    _7: u8
    _8: u32
    _9: u32
    _10: bool
    _11: u8
    _12: u32
    _13: u32
    _14: bool
  blocks:
    bb0:
      _2 = cast _1 : u32 -> u8;
      _5 = cast 8:i32 : i32 -> u32;
      _6 = _5 < 32:u32 : u32;
      assert _6 then bb1 else unwind;
    bb1:
      _4 = _1 >> 8:i32 : u32;
      _3 = cast _4 : u32 -> u8;
      _9 = cast 16:i32 : i32 -> u32;
      _10 = _9 < 32:u32 : u32;
      assert _10 then bb2 else unwind;
    bb2:
      _8 = _1 >> 16:i32 : u32;
      _7 = cast _8 : u32 -> u8;
      _13 = cast 24:i32 : i32 -> u32;
      _14 = _13 < 32:u32 : u32;
      assert _14 then bb3 else unwind;
    bb3:
      _12 = _1 >> 24:i32 : u32;
      _11 = cast _12 : u32 -> u8;
      _0 = (_2, _3, _7, _11);
      return;
}

// Pair: src_to_pair -> tgt_to_pair
// VERIFIED: tgt refines src.

fn src_to_pair(_1: u32) -> (u16, u16) {
  locals:
    _0: (u16, u16)
    _1: u32
  blocks:
    bb0:
      _0 = transmute _1 : u32 -> (u16, u16);
      return;
}

fn tgt_to_pair(_1: u32) -> (u16, u16) {
  locals:
    _0: (u16, u16)
    _1: u32
    _2: u16
    _3: u32
    _4: u16
    _5: u32
    _6: u32
    _7: bool
  blocks:
    bb0:
      _3 = _1 & 65535:u32 : u32;
      _2 = cast _3 : u32 -> u16;
      _6 = cast 16:i32 : i32 -> u32;
      _7 = _6 < 32:u32 : u32;
      assert _7 then bb1 else unwind;
    bb1:
      _5 = _1 >> 16:i32 : u32;
      _4 = cast _5 : u32 -> u16;
      _0 = (_2, _4);
      return;
}

// Compiler diagnostics:
// warning: unnecessary transmute
//   --> ui-tests/casts/transmute.rs:30:14
//    |
// 30 |     unsafe { std::mem::transmute(x) }
//    |              ^^^^^^^^^^^^^^^^^^^^^^
//    |
//    = help: there's also `to_le_bytes` and `to_be_bytes` if you expect a particular byte order
//    = note: `#[warn(unnecessary_transmutes)]` on by default
// help: replace this with
//    |
// 30 -     unsafe { std::mem::transmute(x) }
// 30 +     unsafe { u32::to_ne_bytes(x) }
//    |
//
// warning: 1 warning emitted
//
