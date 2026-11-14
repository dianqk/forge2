// Exit status: 2

// Pair: src -> tgt
// ERROR
// return type mismatch

fn src(_1: u8) -> u8 {
  locals:
    _0: u8
    _1: u8
  blocks:
    bb0:
      _0 = _1;
      return;
}

fn tgt(_1: u8) -> u16 {
  locals:
    _0: u16
    _1: u8
  blocks:
    bb0:
      _0 = cast _1 : u8 -> u16;
      return;
}
