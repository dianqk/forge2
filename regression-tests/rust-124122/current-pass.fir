// Exit status: 0

// Pass: MatchBranchSimplification

// Function: signed_first
// VERIFIED: after refines before.

fn signed_first_before(_1: i8) -> i16 {
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

fn signed_first_after(_1: i8) -> i16 {
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

// Function: signed_later
// VERIFIED: after refines before.

fn signed_later_before(_1: i8) -> i16 {
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
      _0 = 253:i16;
      goto bb5;
    bb3:
      _0 = 2:i16;
      goto bb5;
    bb4:
      _0 = -1:i16;
      goto bb5;
    bb5:
      return;
}

fn signed_later_after(_1: i8) -> i16 {
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
      _0 = 253:i16;
      goto bb5;
    bb3:
      _0 = 2:i16;
      goto bb5;
    bb4:
      _0 = -1:i16;
      goto bb5;
    bb5:
      return;
}
