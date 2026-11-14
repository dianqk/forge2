// Exit status: 1

// Pair: src_a -> tgt_a
// VERIFIED: tgt refines src.

fn src_a(_1: i8, _2: i8) -> i8 {
  locals:
    _0: i8
    _1: i8
    _2: i8
    _3: i8
  blocks:
    bb0:
      _3 = _1;
      _1 = _3 + _2 : i8;
      _0 = _1;
      return;
}

fn tgt_a(_1: i8, _2: i8) -> i8 {
  locals:
    _0: i8
    _1: i8
    _2: i8
  blocks:
    bb0:
      _0 = _1 + _2 : i8;
      return;
}

// Pair: src_b -> tgt_b
// VERIFIED: tgt refines src.

fn src_b(_1: i8, _2: i8) -> i8 {
  locals:
    _0: i8
    _1: i8
    _2: i8
    _3: &mut i8
    _4: i8
  blocks:
    bb0:
      _3 = &mut _1;
      _4 = *_3;
      *_3 = _4 + _2 : i8;
      _0 = *_3;
      return;
}

fn tgt_b(_1: i8, _2: i8) -> i8 {
  locals:
    _0: i8
    _1: i8
    _2: i8
  blocks:
    bb0:
      _0 = _1 + _2 : i8;
      return;
}

// Pair: src_c -> tgt_c
// VERIFIED: tgt refines src.

fn src_c(_1: (i8, i8), _2: i8) -> i8 {
  locals:
    _0: i8
    _1: (i8, i8)
    _2: i8
    _3: &mut i8
    _4: i8
  blocks:
    bb0:
      _3 = &mut _1.0;
      _4 = *_3;
      *_3 = _4 + _2 : i8;
      _0 = _1.0;
      return;
}

fn tgt_c(_1: (i8, i8), _2: i8) -> i8 {
  locals:
    _0: i8
    _1: (i8, i8)
    _2: i8
    _3: i8
  blocks:
    bb0:
      _3 = _1.0;
      _0 = _3 + _2 : i8;
      return;
}

// Pair: src_d -> tgt_d
// VERIFIED: tgt refines src.

fn src_d(_1: &mut i8, _2: i8) -> i8 {
  locals:
    _0: i8
    _1: &mut i8
    _2: i8
    _3: i8
  blocks:
    bb0:
      _3 = *_1;
      *_1 = _3 + _2 : i8;
      _0 = *_1;
      return;
}

fn tgt_d(_1: &mut i8, _2: i8) -> i8 {
  locals:
    _0: i8
    _1: &mut i8
    _2: i8
    _3: i8
  blocks:
    bb0:
      _3 = *_1;
      *_1 = _2 + _3 : i8;
      _0 = *_1;
      return;
}

// Pair: src_ref -> tgt_ref
// COUNTEREXAMPLE
// Reference parameter mutation mismatch:
//   _1 (a) = &mut 0
//   _2 (b) = 0
//   src writes through a: 0
//   tgt writes through a: 1

fn src_ref(_1: &mut i8, _2: i8) -> i8 {
  locals:
    _0: i8
    _1: &mut i8
    _2: i8
  blocks:
    bb0:
      *_1 = _2;
      _0 = _2;
      return;
}

fn tgt_ref(_1: &mut i8, _2: i8) -> i8 {
  locals:
    _0: i8
    _1: &mut i8
    _2: i8
  blocks:
    bb0:
      *_1 = _2 + 1:i8 : i8;
      _0 = _2;
      return;
}

// Pair: src_ref_2 -> tgt_ref_2
// VERIFIED: tgt refines src.

fn src_ref_2(_1: &mut i8, _2: &mut i8) -> i8 {
  locals:
    _0: i8
    _1: &mut i8
    _2: &mut i8
    _3: i8
    _4: i8
  blocks:
    bb0:
      *_1 = 1:i8;
      *_2 = 2:i8;
      _3 = *_1;
      _4 = *_2;
      _0 = _3 + _4 : i8;
      return;
}

fn tgt_ref_2(_1: &mut i8, _2: &mut i8) -> i8 {
  locals:
    _0: i8
    _1: &mut i8
    _2: &mut i8
  blocks:
    bb0:
      *_1 = 1:i8;
      *_2 = 2:i8;
      _0 = 3:i8;
      return;
}
