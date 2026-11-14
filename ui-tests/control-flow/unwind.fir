// Exit status: 1

// Pair: src_source_unwind -> tgt_source_unwind
// COUNTEREXAMPLE
// Source unwinds where target does not.
//   _1 (xs) = (0, 0, 0, 0)
//   _2 (i) = 4

fn src_source_unwind(_1: (i8, i8, i8, i8), _2: u64) -> i8 {
  locals:
    _0: i8
    _1: (i8, i8, i8, i8)
    _2: u64
    _3: bool
  blocks:
    bb0:
      _3 = _2 < 4:u64 : u64;
      assert _3 then bb1 else unwind;
    bb1:
      _0 = _1[_2];
      return;
}

fn tgt_source_unwind(_1: (i8, i8, i8, i8), _2: u64) -> i8 {
  locals:
    _0: i8
    _1: (i8, i8, i8, i8)
    _2: u64
    _3: u64
    _4: bool
  blocks:
    bb0:
      _3 = _2 & 3:u64 : u64;
      _4 = _3 < 4:u64 : u64;
      assert _4 then bb1 else unwind;
    bb1:
      _0 = _1[_3];
      return;
}

// Pair: src_target_unwind -> tgt_target_unwind
// COUNTEREXAMPLE
// Target unwinds where source does not.
//   _1 (xs) = (0, 0, 0, 0)
//   _2 (i) = 4

fn src_target_unwind(_1: (i8, i8, i8, i8), _2: u64) -> i8 {
  locals:
    _0: i8
    _1: (i8, i8, i8, i8)
    _2: u64
    _3: u64
    _4: bool
  blocks:
    bb0:
      _3 = _2 & 3:u64 : u64;
      _4 = _3 < 4:u64 : u64;
      assert _4 then bb1 else unwind;
    bb1:
      _0 = _1[_3];
      return;
}

fn tgt_target_unwind(_1: (i8, i8, i8, i8), _2: u64) -> i8 {
  locals:
    _0: i8
    _1: (i8, i8, i8, i8)
    _2: u64
    _3: bool
  blocks:
    bb0:
      _3 = _2 < 4:u64 : u64;
      assert _3 then bb1 else unwind;
    bb1:
      _0 = _1[_2];
      return;
}

// Pair: src_unwind_reason -> tgt_unwind_reason
// VERIFIED: tgt refines src.

fn src_unwind_reason(_1: (i8, i8, i8, i8), _2: u64) -> i8 {
  locals:
    _0: i8
    _1: (i8, i8, i8, i8)
    _2: u64
    _3: u64
    _4: (u64, bool)
    _5: bool
  blocks:
    bb0:
      _4 = _2 + overflow 4:u64 : u64;
      assert !_4.1 then bb1 else unwind;
    bb1:
      _3 = _4.0;
      _5 = _3 < 4:u64 : u64;
      assert _5 then bb2 else unwind;
    bb2:
      _0 = _1[_3];
      return;
}

fn tgt_unwind_reason(_1: (i8, i8, i8, i8), _2: u64) -> i8 {
  locals:
    _0: i8
    _1: (i8, i8, i8, i8)
    _2: u64
    _3: u64
    _4: (u64, bool)
    _5: bool
  blocks:
    bb0:
      _4 = _2 + overflow 5:u64 : u64;
      assert !_4.1 then bb1 else unwind;
    bb1:
      _3 = _4.0;
      _5 = _3 < 4:u64 : u64;
      assert _5 then bb2 else unwind;
    bb2:
      _0 = _1[_3];
      return;
}
