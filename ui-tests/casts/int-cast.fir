// Exit status: 1

// Pair: src -> tgt
// COUNTEREXAMPLE
// Return value mismatch:
//   _1 (i) = -1
//   src returns: 255
//   tgt returns: -1

fn src(_1: i8) -> i16 {
  locals:
    _0: i16
    _1: i8
  blocks:
    bb0:
      switchInt _1 [255 => bb4, 2 => bb3, 253 => bb2] otherwise bb1;
    bb1:
      _0 = cast _1 : i8 -> i16;
      goto bb5;
    bb2:
      _0 = -3:i16;
      goto bb5;
    bb3:
      _0 = 2:i16;
      goto bb5;
    bb4:
      _0 = 255:i16;
      goto bb5;
    bb5:
      return;
}

fn tgt(_1: i8) -> i16 {
  locals:
    _0: i16
    _1: i8
  blocks:
    bb0:
      _0 = cast _1 : i8 -> i16;
      return;
}
