// Exit status: 1

// Pair: src_a -> tgt_a
// COUNTEREXAMPLE
// Return value mismatch:
//   _1 (i) = Foo::A (-1)
//   src returns: 255
//   tgt returns: -1

fn src_a(_1: i8) -> i16 {
  locals:
    _0: i16
    _1: i8
    _2: i8
  blocks:
    bb0:
      assume _1 in [-1:i8, 2:i8, -3:i8];
      _2 = _1;
      switchInt _2 [255 => bb4, 2 => bb3, 253 => bb2] otherwise bb1;
    bb1:
      unreachable;
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

fn tgt_a(_1: i8) -> i16 {
  locals:
    _0: i16
    _1: i8
    _2: i8
  blocks:
    bb0:
      assume _1 in [-1:i8, 2:i8, -3:i8];
      _2 = _1;
      _0 = cast _2 : i8 -> i16;
      return;
}

// Pair: src_b -> tgt_b
// VERIFIED: tgt refines src.

fn src_b(_1: i8) -> i16 {
  locals:
    _0: i16
    _1: i8
    _2: i8
  blocks:
    bb0:
      assume _1 in [1:i8, 2:i8, -3:i8];
      _2 = _1;
      switchInt _2 [255 => bb5, 1 => bb4, 2 => bb3, 253 => bb2] otherwise bb1;
    bb1:
      _0 = 0:i16;
      goto bb6;
    bb2:
      _0 = -3:i16;
      goto bb6;
    bb3:
      _0 = 2:i16;
      goto bb6;
    bb4:
      _0 = 1:i16;
      goto bb6;
    bb5:
      _0 = 255:i16;
      goto bb6;
    bb6:
      return;
}

fn tgt_b(_1: i8) -> i16 {
  locals:
    _0: i16
    _1: i8
    _2: i8
  blocks:
    bb0:
      assume _1 in [1:i8, 2:i8, -3:i8];
      _2 = _1;
      _0 = cast _2 : i8 -> i16;
      return;
}
