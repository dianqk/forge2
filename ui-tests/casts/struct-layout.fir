// Exit status: 1

// Pair: src -> tgt
// COUNTEREXAMPLE
// Return value mismatch:
//   _1 (x) = (255, 255, 0)
//   src returns: 16711935
//   tgt returns: 65535

fn src(_1: (u8, u16, u8)) -> u32 {
  locals:
    _0: u32
    _1: (u8, u16, u8)
  blocks:
    bb0:
      _0 = transmute _1 : (u8, u16, u8) -> u32;
      return;
}

fn tgt(_1: (u8, u16, u8)) -> u32 {
  locals:
    _0: u32
    _1: (u8, u16, u8)
    _2: u32
    _3: u32
    _4: u8
    _5: u32
    _6: u32
    _7: u16
    _8: u32
    _9: u32
    _10: u8
  blocks:
    bb0:
      _4 = _1.0;
      _3 = cast _4 : u8 -> u32;
      _7 = _1.1;
      _6 = cast _7 : u16 -> u32;
      _5 = _6 << 8:i32 : u32;
      _2 = _3 | _5 : u32;
      _10 = _1.2;
      _9 = cast _10 : u8 -> u32;
      _8 = _9 << 24:i32 : u32;
      _0 = _2 | _8 : u32;
      return;
}

// Pair: src_control -> tgt_control
// VERIFIED: tgt refines src.

fn src_control(_1: (u8, u16, u8)) -> u32 {
  locals:
    _0: u32
    _1: (u8, u16, u8)
  blocks:
    bb0:
      _0 = transmute _1 : (u8, u16, u8) -> u32;
      return;
}

fn tgt_control(_1: (u8, u16, u8)) -> u32 {
  locals:
    _0: u32
    _1: (u8, u16, u8)
    _2: u32
    _3: u32
    _4: u16
    _5: u32
    _6: u32
    _7: u8
    _8: u32
    _9: u32
    _10: u8
  blocks:
    bb0:
      _4 = _1.1;
      _3 = cast _4 : u16 -> u32;
      _7 = _1.0;
      _6 = cast _7 : u8 -> u32;
      _5 = _6 << 16:i32 : u32;
      _2 = _3 | _5 : u32;
      _10 = _1.2;
      _9 = cast _10 : u8 -> u32;
      _8 = _9 << 24:i32 : u32;
      _0 = _2 | _8 : u32;
      return;
}
