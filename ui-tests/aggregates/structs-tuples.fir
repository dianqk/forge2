// Exit status: 1

// Pair: src_const_struct -> tgt_const_struct
// NOTE: source FIR is identical to target FIR.
// VERIFIED: tgt refines src.

fn src_const_struct() -> (i32, i8) {
  locals:
    _0: (i32, i8)
  blocks:
    bb0:
      _0 = (7:i32, -3:i8);
      return;
}

fn tgt_const_struct() -> (i32, i8) {
  locals:
    _0: (i32, i8)
  blocks:
    bb0:
      _0 = (7:i32, -3:i8);
      return;
}

// Pair: src_const_struct_mismatch -> tgt_const_struct_mismatch
// COUNTEREXAMPLE
// Return value mismatch:
//   src returns: (7, -3)
//   tgt returns: (7, 0)

fn src_const_struct_mismatch() -> (i32, i8) {
  locals:
    _0: (i32, i8)
  blocks:
    bb0:
      _0 = (7:i32, -3:i8);
      return;
}

fn tgt_const_struct_mismatch() -> (i32, i8) {
  locals:
    _0: (i32, i8)
  blocks:
    bb0:
      _0 = (7:i32, 0:i8);
      return;
}

// Pair: src_local -> tgt_local
// NOTE: source FIR is identical to target FIR.
// VERIFIED: tgt refines src.

fn src_local(_1: (i32, i8)) -> i32 {
  locals:
    _0: i32
    _1: (i32, i8)
  blocks:
    bb0:
      _0 = _1.0;
      return;
}

fn tgt_local(_1: (i32, i8)) -> i32 {
  locals:
    _0: i32
    _1: (i32, i8)
  blocks:
    bb0:
      _0 = _1.0;
      return;
}

// Pair: src_nest -> tgt_nest
// COUNTEREXAMPLE
// Return value mismatch:
//   _1 (x) = ((-1, 0), (0, 0))
//   src returns: (-1, 0)
//   tgt returns: (0, 0)

fn src_nest(_1: ((i32, i8), (i32, i8))) -> (i32, i8) {
  locals:
    _0: (i32, i8)
    _1: ((i32, i8), (i32, i8))
  blocks:
    bb0:
      _0 = _1.0;
      return;
}

fn tgt_nest(_1: ((i32, i8), (i32, i8))) -> (i32, i8) {
  locals:
    _0: (i32, i8)
    _1: ((i32, i8), (i32, i8))
  blocks:
    bb0:
      _0 = _1.1;
      return;
}

// Pair: src_ret -> tgt_ret
// COUNTEREXAMPLE
// Return value mismatch:
//   _1 (x) = 0
//   _2 (y) = -1
//   src returns: (0, -1)
//   tgt returns: (0, 0)

fn src_ret(_1: i32, _2: i8) -> (i32, i8) {
  locals:
    _0: (i32, i8)
    _1: i32
    _2: i8
  blocks:
    bb0:
      _0 = (_1, _2);
      return;
}

fn tgt_ret(_1: i32, _2: i8) -> (i32, i8) {
  locals:
    _0: (i32, i8)
    _1: i32
    _2: i8
  blocks:
    bb0:
      _0 = (_1, 0:i8);
      return;
}

// Pair: src_struct -> tgt_struct
// NOTE: source FIR is identical to target FIR.
// VERIFIED: tgt refines src.

fn src_struct(_1: (i32, i8)) -> i32 {
  locals:
    _0: i32
    _1: (i32, i8)
  blocks:
    bb0:
      _0 = _1.0;
      return;
}

fn tgt_struct(_1: (i32, i8)) -> i32 {
  locals:
    _0: i32
    _1: (i32, i8)
  blocks:
    bb0:
      _0 = _1.0;
      return;
}

// Pair: src_tuple -> tgt_tuple
// NOTE: source FIR is identical to target FIR.
// VERIFIED: tgt refines src.

fn src_tuple(_1: (i32, i8)) -> i32 {
  locals:
    _0: i32
    _1: (i32, i8)
  blocks:
    bb0:
      _0 = _1.0;
      return;
}

fn tgt_tuple(_1: (i32, i8)) -> i32 {
  locals:
    _0: i32
    _1: (i32, i8)
  blocks:
    bb0:
      _0 = _1.0;
      return;
}
