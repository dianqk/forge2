// Exit status: 1

// Pair: src -> tgt
// COUNTEREXAMPLE
// Return value mismatch:
//   _1 (param_1) = -1
//   src returns: 255
//   tgt returns: -1

fn src(_1: i8) -> i16 {
  locals:
    _0: i16
    _1: i8
  blocks:
    bb0:
      switchInt _1 [255 => bb1, 2 => bb2, 253 => bb3] otherwise bb4;
    bb1:
      _0 = 255:i16;
      return;
    bb2:
      _0 = 2:i16;
      return;
    bb3:
      _0 = -3:i16;
      return;
    bb4:
      unreachable;
}

fn tgt(_1: i8) -> i16 {
  locals:
    _0: i16
    _1: i8
  blocks:
    bb0:
      switchInt _1 [255 => bb1, 2 => bb2, 253 => bb3] otherwise bb4;
    bb1:
      _0 = -1:i16;
      return;
    bb2:
      _0 = 2:i16;
      return;
    bb3:
      _0 = -3:i16;
      return;
    bb4:
      unreachable;
}
