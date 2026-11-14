// Exit status: 1

// Pair: src -> tgt
// COUNTEREXAMPLE
// Return value mismatch:
//   _1 (u) = 246
//   src returns: 9
//   tgt returns: 246

fn src(_1: u8) -> u8 {
  locals:
    _0: u8
    _1: u8
  blocks:
    bb0:
      _1 = 9:u8;
      _0 = _1;
      return;
}

fn tgt(_1: u8) -> u8 {
  locals:
    _0: u8
    _1: u8
  blocks:
    bb0:
      _0 = _1;
      return;
}

// Pair: src_construct -> tgt_construct
// VERIFIED: tgt refines src.

fn src_construct(_1: u8) -> u8 {
  locals:
    _0: u8
    _1: u8
    _2: u8
  blocks:
    bb0:
      _2 = _1;
      _0 = _2;
      return;
}

fn tgt_construct(_1: u8) -> u8 {
  locals:
    _0: u8
    _1: u8
  blocks:
    bb0:
      _0 = _1;
      return;
}

// Pair: src_control -> tgt_control
// VERIFIED: tgt refines src.

fn src_control(_1: u8) -> u8 {
  locals:
    _0: u8
    _1: u8
  blocks:
    bb0:
      _1 = 9:u8;
      _0 = _1;
      return;
}

fn tgt_control(_1: u8) -> u8 {
  locals:
    _0: u8
    _1: u8
  blocks:
    bb0:
      _0 = 9:u8;
      return;
}

// Pair: src_signed -> tgt_signed
// VERIFIED: tgt refines src.

fn src_signed(_1: u32) -> i32 {
  locals:
    _0: i32
    _1: u32
    _2: u32
  blocks:
    bb0:
      _2 = 0:u32;
      _2 = _1;
      _0 = _2;
      return;
}

fn tgt_signed(_1: u32) -> i32 {
  locals:
    _0: i32
    _1: u32
  blocks:
    bb0:
      _0 = cast _1 : u32 -> i32;
      return;
}
