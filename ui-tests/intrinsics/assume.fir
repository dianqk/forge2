// Exit status: 1

// Pair: src -> tgt
// VERIFIED: tgt refines src.

fn src(_1: i32) -> i32 {
  locals:
    _0: i32
    _1: i32
    _2: bool
  blocks:
    bb0:
      _2 = _1 > 0:i32 : i32;
      assume _2;
      _0 = _1;
      return;
}

fn tgt(_1: i32) -> i32 {
  locals:
    _0: i32
    _1: i32
  blocks:
    bb0:
      _0 = _1;
      return;
}

// Pair: src_rev -> tgt_rev
// COUNTEREXAMPLE
// Target reaches unreachable where source does not.
//   _1 (x) = 0

fn src_rev(_1: i32) -> i32 {
  locals:
    _0: i32
    _1: i32
  blocks:
    bb0:
      _0 = _1;
      return;
}

fn tgt_rev(_1: i32) -> i32 {
  locals:
    _0: i32
    _1: i32
    _2: bool
  blocks:
    bb0:
      _2 = _1 > 0:i32 : i32;
      assume _2;
      _0 = _1;
      return;
}

// Pair: src_ub -> tgt_ub
// COUNTEREXAMPLE
// Source always reaches unreachable.

fn src_ub(_1: i32) -> i32 {
  locals:
    _0: i32
    _1: i32
    _2: bool
    _3: bool
  blocks:
    bb0:
      _2 = _1 > 0:i32 : i32;
      assume _2;
      _3 = _1 < 0:i32 : i32;
      assume _3;
      _0 = _1;
      return;
}

fn tgt_ub(_1: i32) -> i32 {
  locals:
    _0: i32
    _1: i32
  blocks:
    bb0:
      _0 = _1 + 1:i32 : i32;
      return;
}
