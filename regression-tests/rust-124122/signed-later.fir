// Exit status: 1

// Pass: MatchBranchSimplification_124122

// Function: signed_later
// COUNTEREXAMPLE
// Return value mismatch:
//   _1 (i) = Input::C (-3)
//   before returns: 253
//   after returns: -3

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
    _3: i8
  blocks:
    bb0:
      assume _1 in [-1:i8, 2:i8, -3:i8];
      _2 = _1;
      _3 = _2;
      _0 = cast _3 : i8 -> i16;
      goto bb5;
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
