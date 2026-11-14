// Exit status: 1

// Pair: src_a -> tgt_a
// COUNTEREXAMPLE
// Return value mismatch:
//   _1 (i) = 0
//   src returns: 0
//   tgt returns: -1

fn src_a(_1: i8) -> i16 {
  locals:
    _0: i16
    _1: i8
  blocks:
    bb0:
      _0 = cast _1 : i8 -> i16;
      return;
}

fn tgt_a(_1: i8) -> i16 {
  locals:
    _0: i16
    _1: i8
    _2: i16
  blocks:
    bb0:
      _2 = cast _1 : i8 -> i16;
      _0 = !_2;
      return;
}

// Pair: src_b -> tgt_b
// NOTE: source FIR is identical to target FIR.
// VERIFIED: tgt refines src.

fn src_b(_1: u8) -> u8 {
  locals:
    _0: u8
    _1: u8
  blocks:
    bb0:
      _0 = !_1;
      return;
}

fn tgt_b(_1: u8) -> u8 {
  locals:
    _0: u8
    _1: u8
  blocks:
    bb0:
      _0 = !_1;
      return;
}
