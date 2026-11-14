// Exit status: 101

// Process diagnostics:
//
// not implemented: Failed to translate tgt to Forge IR:
// bb0 terminator _0 = bswap(move _1) -> [return: bb1, unwind continue]: unsupported call terminator: callee=bswap, func=bswap, func_ty=<unknown: unsupported constant operand type: FnDef(DefId(0:3 ~ bswap_shadow[893f]::bswap), Binder { value: [], bound_vars: [] })>, args=[move _1], destination=_0, target=Some("bb1"), unwind=Continue
