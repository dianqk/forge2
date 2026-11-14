// Exit status: 0

// Pair: src -> tgt
// NOTE: source FIR is identical to target FIR.
// VERIFIED: tgt refines src.

fn src(_1: i64, _2: i64) -> i64 {
  locals:
    _0: i64
    _1: i64
    _2: i64
    _3: bool
    _4: bool
    _5: bool
    _6: bool
  blocks:
    bb0:
      _3 = _2 == 0:i64 : i64;
      assert !_3 then bb1 else unwind;
    bb1:
      _4 = _2 == -1:i64 : i64;
      _5 = _1 == -9223372036854775808:i64 : i64;
      _6 = _4 & _5 : bool;
      assert !_6 then bb2 else unwind;
    bb2:
      _0 = _1 / _2 : i64;
      return;
}

fn tgt(_1: i64, _2: i64) -> i64 {
  locals:
    _0: i64
    _1: i64
    _2: i64
    _3: bool
    _4: bool
    _5: bool
    _6: bool
  blocks:
    bb0:
      _3 = _2 == 0:i64 : i64;
      assert !_3 then bb1 else unwind;
    bb1:
      _4 = _2 == -1:i64 : i64;
      _5 = _1 == -9223372036854775808:i64 : i64;
      _6 = _4 & _5 : bool;
      assert !_6 then bb2 else unwind;
    bb2:
      _0 = _1 / _2 : i64;
      return;
}

// Pair: src_rem -> tgt_rem
// NOTE: source FIR is identical to target FIR.
// VERIFIED: tgt refines src.

fn src_rem(_1: i64, _2: i64) -> i64 {
  locals:
    _0: i64
    _1: i64
    _2: i64
    _3: bool
    _4: bool
    _5: bool
    _6: bool
  blocks:
    bb0:
      _3 = _2 == 0:i64 : i64;
      assert !_3 then bb1 else unwind;
    bb1:
      _4 = _2 == -1:i64 : i64;
      _5 = _1 == -9223372036854775808:i64 : i64;
      _6 = _4 & _5 : bool;
      assert !_6 then bb2 else unwind;
    bb2:
      _0 = _1 % _2 : i64;
      return;
}

fn tgt_rem(_1: i64, _2: i64) -> i64 {
  locals:
    _0: i64
    _1: i64
    _2: i64
    _3: bool
    _4: bool
    _5: bool
    _6: bool
  blocks:
    bb0:
      _3 = _2 == 0:i64 : i64;
      assert !_3 then bb1 else unwind;
    bb1:
      _4 = _2 == -1:i64 : i64;
      _5 = _1 == -9223372036854775808:i64 : i64;
      _6 = _4 & _5 : bool;
      assert !_6 then bb2 else unwind;
    bb2:
      _0 = _1 % _2 : i64;
      return;
}
