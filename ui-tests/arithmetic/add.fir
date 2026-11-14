// Exit status: 1

// Pair: src -> tgt
// COUNTEREXAMPLE
// Target reaches unreachable where source does not.
//   _1 (a) = -122
//   _2 (b) = -122

fn src(_1: i8, _2: i8) -> i8 {
  locals:
    _0: i8
    _1: i8
    _2: i8
  blocks:
    bb0:
      _0 = _1 + _2 : i8;
      return;
}

fn tgt(_1: i8, _2: i8) -> i8 {
  locals:
    _0: i8
    _1: i8
    _2: i8
  blocks:
    bb0:
      _0 = _1 + unchecked _2 : i8;
      return;
}

// Pair: src_overflowing -> tgt_overflowing
// NOTE: source FIR is identical to target FIR.
// VERIFIED: tgt refines src.

fn src_overflowing(_1: i8, _2: i8) -> (i8, bool) {
  locals:
    _0: (i8, bool)
    _1: i8
    _2: i8
  blocks:
    bb0:
      _0 = _1 + overflow _2 : i8;
      return;
}

fn tgt_overflowing(_1: i8, _2: i8) -> (i8, bool) {
  locals:
    _0: (i8, bool)
    _1: i8
    _2: i8
  blocks:
    bb0:
      _0 = _1 + overflow _2 : i8;
      return;
}
