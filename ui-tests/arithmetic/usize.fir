// Exit status: 0

// Pair: src -> tgt
// VERIFIED: tgt refines src.

fn src(_1: u64, _2: u64) -> u64 {
  locals:
    _0: u64
    _1: u64
    _2: u64
  blocks:
    bb0:
      _0 = _1 + _2 : u64;
      return;
}

fn tgt(_1: u64, _2: u64) -> u64 {
  locals:
    _0: u64
    _1: u64
    _2: u64
  blocks:
    bb0:
      _0 = _2 + _1 : u64;
      return;
}
