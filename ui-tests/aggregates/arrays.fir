// Exit status: 0

// Pair: src_arg -> tgt_arg
// NOTE: source FIR is identical to target FIR.
// VERIFIED: tgt refines src.

fn src_arg(_1: (i8, i8, i8)) -> i8 {
  locals:
    _0: i8
    _1: (i8, i8, i8)
  blocks:
    bb0:
      _0 = _1.1;
      return;
}

fn tgt_arg(_1: (i8, i8, i8)) -> i8 {
  locals:
    _0: i8
    _1: (i8, i8, i8)
  blocks:
    bb0:
      _0 = _1.1;
      return;
}

// Pair: src_index -> tgt_index
// NOTE: source FIR is identical to target FIR.
// VERIFIED: tgt refines src.

fn src_index(_1: (i8, i8, i8, i8), _2: u64) -> i8 {
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

fn tgt_index(_1: (i8, i8, i8, i8), _2: u64) -> i8 {
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

// Pair: src_local -> tgt_local
// VERIFIED: tgt refines src.

fn src_local(_1: i8, _2: i8, _3: i8) -> (i8, i8, i8) {
  locals:
    _0: (i8, i8, i8)
    _1: i8
    _2: i8
    _3: i8
    _4: i8
  blocks:
    bb0:
      _0 = (_1, _2, _3);
      _4 = _0.0;
      _0.1 = _4;
      return;
}

fn tgt_local(_1: i8, _2: i8, _3: i8) -> (i8, i8, i8) {
  locals:
    _0: (i8, i8, i8)
    _1: i8
    _2: i8
    _3: i8
  blocks:
    bb0:
      _0 = (_1, _1, _3);
      return;
}

// Pair: src_nested -> tgt_nested
// NOTE: source FIR is identical to target FIR.
// VERIFIED: tgt refines src.

fn src_nested(_1: ((i8, i8), (i8, i8))) -> i8 {
  locals:
    _0: i8
    _1: ((i8, i8), (i8, i8))
  blocks:
    bb0:
      _0 = _1.1.0;
      return;
}

fn tgt_nested(_1: ((i8, i8), (i8, i8))) -> i8 {
  locals:
    _0: i8
    _1: ((i8, i8), (i8, i8))
  blocks:
    bb0:
      _0 = _1.1.0;
      return;
}

// Pair: src_store -> tgt_store
// NOTE: source FIR is identical to target FIR.
// VERIFIED: tgt refines src.

fn src_store(_1: (i8, i8, i8, i8), _2: u64, _3: i8) -> (i8, i8, i8, i8) {
  locals:
    _0: (i8, i8, i8, i8)
    _1: (i8, i8, i8, i8)
    _2: u64
    _3: i8
    _4: bool
  blocks:
    bb0:
      _4 = _2 < 4:u64 : u64;
      assert _4 then bb1 else unwind;
    bb1:
      _1[_2] = _3;
      _0 = _1;
      return;
}

fn tgt_store(_1: (i8, i8, i8, i8), _2: u64, _3: i8) -> (i8, i8, i8, i8) {
  locals:
    _0: (i8, i8, i8, i8)
    _1: (i8, i8, i8, i8)
    _2: u64
    _3: i8
    _4: bool
  blocks:
    bb0:
      _4 = _2 < 4:u64 : u64;
      assert _4 then bb1 else unwind;
    bb1:
      _1[_2] = _3;
      _0 = _1;
      return;
}
