// Exit status: 0

// Pass: MatchBranchSimplification_124122

// Function: boolean_match
// VERIFIED: after refines before.

fn boolean_match_before(_1: u8) -> bool {
  locals:
    _0: bool
    _1: u8
  blocks:
    bb0:
      switchInt _1 [42 => bb2] otherwise bb1;
    bb1:
      _0 = 0:bool;
      goto bb3;
    bb2:
      _0 = 1:bool;
      goto bb3;
    bb3:
      return;
}

fn boolean_match_after(_1: u8) -> bool {
  locals:
    _0: bool
    _1: u8
    _2: u8
  blocks:
    bb0:
      _2 = _1;
      _0 = _2 == 42:u8 : u8;
      goto bb3;
    bb1:
      _0 = 0:bool;
      goto bb3;
    bb2:
      _0 = 1:bool;
      goto bb3;
    bb3:
      return;
}

// Function: signed_widening
// VERIFIED: after refines before.

fn signed_widening_before(_1: i8) -> i16 {
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
      _0 = -1:i16;
      goto bb5;
    bb5:
      return;
}

fn signed_widening_after(_1: i8) -> i16 {
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
      _0 = -3:i16;
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

// Function: unsigned_widening
// VERIFIED: after refines before.

fn unsigned_widening_before(_1: u8) -> u16 {
  locals:
    _0: u16
    _1: u8
    _2: u8
  blocks:
    bb0:
      assume _1 in [255:u8, 2:u8, 253:u8];
      _2 = _1;
      switchInt _2 [255 => bb4, 2 => bb3, 253 => bb2] otherwise bb1;
    bb1:
      unreachable;
    bb2:
      _0 = 253:u16;
      goto bb5;
    bb3:
      _0 = 2:u16;
      goto bb5;
    bb4:
      _0 = 255:u16;
      goto bb5;
    bb5:
      return;
}

fn unsigned_widening_after(_1: u8) -> u16 {
  locals:
    _0: u16
    _1: u8
    _2: u8
    _3: u8
  blocks:
    bb0:
      assume _1 in [255:u8, 2:u8, 253:u8];
      _2 = _1;
      _3 = _2;
      _0 = cast _3 : u8 -> u16;
      goto bb5;
    bb1:
      unreachable;
    bb2:
      _0 = 253:u16;
      goto bb5;
    bb3:
      _0 = 2:u16;
      goto bb5;
    bb4:
      _0 = 255:u16;
      goto bb5;
    bb5:
      return;
}
