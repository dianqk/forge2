// Exit status: 1

// Pair: src -> tgt
// COUNTEREXAMPLE
// Reference parameter mutation mismatch:
//   _1 (param_1) = &mut 254
//   src writes through param_1: 1
//   tgt writes through param_1: 254

fn src(_1: &mut u8) -> u8 {
  locals:
    _0: u8
    _1: &mut u8
  blocks:
    bb0:
      *_1 = 1:u8;
      _0 = 0:u8;
      return;
}

fn tgt(_1: &mut u8) -> u8 {
  locals:
    _0: u8
    _1: &mut u8
    _2: u8
  blocks:
    bb0:
      _2 = 1:u8;
      _0 = 0:u8;
      return;
}
