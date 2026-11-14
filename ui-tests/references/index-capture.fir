// Exit status: 1

// Pair: src -> tgt
// COUNTEREXAMPLE
// Return value mismatch:
//   _1 (param_1) = (246, 9)
//   src returns: (9, 9)
//   tgt returns: (246, 9)

fn src(_1: (u8, u8)) -> (u8, u8) {
  locals:
    _0: (u8, u8)
    _1: (u8, u8)
    _2: u64
    _3: &mut u8
  blocks:
    bb0:
      _2 = 0:u64;
      _3 = &mut _1[_2];
      _2 = 1:u64;
      *_3 = 9:u8;
      _0 = _1;
      return;
}

fn tgt(_1: (u8, u8)) -> (u8, u8) {
  locals:
    _0: (u8, u8)
    _1: (u8, u8)
  blocks:
    bb0:
      _1.1 = 9:u8;
      _0 = _1;
      return;
}
