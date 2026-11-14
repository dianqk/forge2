use rustc_middle::mir;
use rustc_middle::ty::util::IntTypeExt;
use rustc_middle::ty::{self, TyCtxt};

use crate::ir::inst::*;
use crate::ir::ty::*;

type TranslateResult<T> = Result<T, String>;

fn pointer_int_width(pointer_width: u32) -> Option<IntWidth> {
    match pointer_width {
        32 => Some(IntWidth::I32),
        64 => Some(IntWidth::I64),
        _ => None,
    }
}

pub fn translate_ty(ty: ty::Ty<'_>, pointer_width: u32) -> Option<ForgeTy> {
    match ty.kind() {
        // FIXME: Model `bool` with 8-bit storage to match Rust layout-sensitive operations
        // such as transmute, but still constrain ordinary bool values to {0, 1}.
        ty::TyKind::Bool => Some(ForgeTy::Int(IntWidth::I8, IntKind::Bool)),
        ty::TyKind::Char => Some(ForgeTy::Int(IntWidth::I32, IntKind::Unsigned)),
        ty::TyKind::Int(int_ty) => {
            let width = match int_ty {
                ty::IntTy::I8 => IntWidth::I8,
                ty::IntTy::I16 => IntWidth::I16,
                ty::IntTy::I32 => IntWidth::I32,
                ty::IntTy::I64 => IntWidth::I64,
                ty::IntTy::I128 => IntWidth::I128,
                ty::IntTy::Isize => pointer_int_width(pointer_width)?,
            };
            Some(ForgeTy::Int(width, IntKind::Signed))
        }
        ty::TyKind::Uint(uint_ty) => {
            let width = match uint_ty {
                ty::UintTy::U8 => IntWidth::I8,
                ty::UintTy::U16 => IntWidth::I16,
                ty::UintTy::U32 => IntWidth::I32,
                ty::UintTy::U64 => IntWidth::I64,
                ty::UintTy::U128 => IntWidth::I128,
                ty::UintTy::Usize => pointer_int_width(pointer_width)?,
            };
            Some(ForgeTy::Int(width, IntKind::Unsigned))
        }
        ty::TyKind::Tuple(fields) => Some(ForgeTy::Aggregate(
            fields
                .iter()
                .map(|f| translate_ty(f, pointer_width))
                .collect::<Option<Vec<_>>>()?,
        )),
        _ => None,
    }
}

pub fn translate_ty_with_tcx<'tcx>(tcx: TyCtxt<'tcx>, ty: ty::Ty<'tcx>) -> Option<ForgeTy> {
    let pointer_width = tcx.data_layout.pointer_size().bits() as u32;

    if let Some(ty) = translate_ty(ty, pointer_width) {
        return Some(ty);
    }

    match ty.kind() {
        ty::TyKind::Array(inner, len) => {
            let len = len.try_to_target_usize(tcx)?.try_into().ok()?;
            let element_ty = translate_ty_with_tcx(tcx, *inner)?;
            Some(ForgeTy::Aggregate(vec![element_ty; len]))
        }
        ty::TyKind::Ref(_, inner, mutability) if *mutability == ty::Mutability::Mut => {
            Some(ForgeTy::Ref(Box::new(translate_ty_with_tcx(tcx, *inner)?)))
        }
        ty::TyKind::Adt(adt_def, _) if adt_def.is_union() => union_storage_ty(tcx, ty),
        ty::TyKind::Adt(adt_def, args) if !adt_def.is_enum() => Some(ForgeTy::Aggregate(
            adt_def
                .all_fields()
                .map(|field| field.ty(tcx, *args).skip_norm_wip())
                .map(|field_ty| translate_ty_with_tcx(tcx, field_ty))
                .collect::<Option<Vec<_>>>()?,
        )),
        ty::TyKind::Adt(adt_def, _) if adt_def.is_enum() => {
            if adt_def
                .variants()
                .iter()
                .any(|variant| !variant.fields.is_empty())
            {
                return None;
            }

            let discr_ty = adt_def.repr().discr_type().to_ty(tcx);
            translate_ty(discr_ty, pointer_width)
        }
        _ => None,
    }
}

/// Full-width integer union fields are alternate views of one allocation.
/// Partial-width/aggregate fields need initialized-byte semantics, so do not
/// admit them through the struct representation.
fn union_storage_ty<'tcx>(tcx: TyCtxt<'tcx>, union_ty: ty::Ty<'tcx>) -> Option<ForgeTy> {
    let ty::TyKind::Adt(adt, args) = union_ty.kind() else {
        return None;
    };
    let typing_env = ty::TypingEnv::fully_monomorphized();
    let storage = tcx
        .layout_of(typing_env.as_query_input(union_ty))
        .ok()?
        .size;
    let width = match storage.bits() {
        8 => IntWidth::I8,
        16 => IntWidth::I16,
        32 => IntWidth::I32,
        64 => IntWidth::I64,
        128 => IntWidth::I128,
        _ => return None,
    };
    for field in adt.all_fields() {
        let field_ty = field.ty(tcx, *args).skip_norm_wip();
        if !matches!(field_ty.kind(), ty::TyKind::Int(_) | ty::TyKind::Uint(_))
            || tcx
                .layout_of(typing_env.as_query_input(field_ty))
                .ok()?
                .size
                != storage
        {
            return None;
        }
    }
    Some(ForgeTy::Int(width, IntKind::Unsigned))
}

fn translate_place<'tcx>(
    tcx: TyCtxt<'tcx>,
    place: &mir::Place<'tcx>,
    body: &mir::Body<'tcx>,
) -> TranslateResult<ForgePlace> {
    let mut projections = Vec::new();
    let mut current_ty = body.local_decls[place.local].ty;

    for projection in place.projection.iter() {
        match projection {
            mir::ProjectionElem::Field(field, field_ty) => {
                if matches!(current_ty.kind(), ty::TyKind::Adt(adt, _) if adt.is_union()) {
                    union_storage_ty(tcx, current_ty)
                        .ok_or_else(|| format!("unsupported union field layout: {current_ty:?}"))?;
                    // Every admitted field covers exactly the same scalar storage.
                    current_ty = field_ty;
                    continue;
                }
                let forge_current = translate_ty_with_tcx(tcx, current_ty)
                    .ok_or_else(|| format!("unsupported projected base type: {:?}", current_ty))?;
                let ForgeTy::Aggregate(fields) = forge_current else {
                    return Err(format!(
                        "field projection on non-aggregate place {:?}: {:?}",
                        place, current_ty
                    ));
                };
                let field_idx = field.as_usize();
                if field_idx >= fields.len() {
                    return Err(format!(
                        "field projection out of bounds in place {:?}",
                        place
                    ));
                }
                projections.push(ForgeProjection::Field(field_idx as u32));
                current_ty = field_ty;
            }
            mir::ProjectionElem::ConstantIndex {
                offset,
                min_length,
                from_end,
            } => {
                let (element_ty, len) = array_projection_info(tcx, current_ty)?;
                let index = translate_constant_index(offset, min_length, from_end, len)?;
                projections.push(ForgeProjection::Field(index as u32));
                current_ty = element_ty;
            }
            mir::ProjectionElem::Index(local) => {
                let (element_ty, _) = array_projection_info(tcx, current_ty)?;
                projections.push(ForgeProjection::Index(VarId(local.as_u32())));
                current_ty = element_ty;
            }
            mir::ProjectionElem::Deref => {
                let ty::TyKind::Ref(_, inner, _) = current_ty.kind() else {
                    return Err(format!(
                        "deref projection on non-reference place {:?}",
                        place
                    ));
                };
                projections.push(ForgeProjection::Deref);
                current_ty = *inner;
            }
            _ => return Err(format!("unsupported place projection: {:?}", place)),
        }
    }

    Ok(ForgePlace {
        base: VarId(place.local.as_u32()),
        projections,
    })
}

fn translate_operand<'tcx>(
    tcx: TyCtxt<'tcx>,
    operand: &mir::Operand<'tcx>,
    body: &mir::Body<'tcx>,
) -> TranslateResult<ForgeOperand> {
    match operand {
        mir::Operand::Copy(place) | mir::Operand::Move(place) => {
            Ok(ForgeOperand::Place(translate_place(tcx, place, body)?))
        }
        mir::Operand::RuntimeChecks(_) => Err("unsupported RuntimeChecks operand".into()),
        mir::Operand::Constant(constant) => {
            let value = translate_constant(tcx, constant.const_, constant.ty())?;
            Ok(ForgeOperand::Const(value))
        }
    }
}

fn translate_constant<'tcx>(
    tcx: TyCtxt<'tcx>,
    constant: mir::Const<'tcx>,
    ty: ty::Ty<'tcx>,
) -> TranslateResult<ForgeConst> {
    match constant {
        mir::Const::Ty(_, value) => translate_ty_const(tcx, value, ty),
        mir::Const::Val(value, _) => translate_const_value(tcx, value, ty),
        mir::Const::Unevaluated(unevaluated, _) => {
            let typing_env = ty::TypingEnv::fully_monomorphized();
            let value = tcx
                .const_eval_resolve(typing_env, unevaluated, rustc_span::DUMMY_SP)
                .map_err(|err| {
                    format!("failed to evaluate constant {:?}: {:?}", unevaluated, err)
                })?;
            translate_const_value(tcx, value, ty)
        }
    }
}

fn translate_const_value<'tcx>(
    tcx: TyCtxt<'tcx>,
    value: mir::ConstValue,
    ty: ty::Ty<'tcx>,
) -> TranslateResult<ForgeConst> {
    if let Some(scalar) = value.try_to_scalar_int() {
        if let Some(field_ty) = scalar_backed_aggregate_field_ty(tcx, ty) {
            return Ok(ForgeConst::Aggregate(vec![translate_scalar_const(
                tcx, field_ty, scalar,
            )?]));
        }
        return translate_scalar_const(tcx, ty, scalar);
    }

    if let Some(destructured) = tcx.try_destructure_mir_constant_for_user_output(value, ty) {
        match ty.kind() {
            ty::TyKind::Adt(adt_def, _) if adt_def.is_enum() => {
                if adt_def
                    .variants()
                    .iter()
                    .any(|variant| !variant.fields.is_empty())
                {
                    return Err(format!(
                        "unsupported enum constant with fields: {:?}",
                        value
                    ));
                }

                let discr_ty = translate_ty_with_tcx(tcx, ty)
                    .ok_or_else(|| format!("unsupported enum constant type: {:?}", ty))?;
                let variant = destructured
                    .variant
                    .ok_or_else(|| format!("missing enum variant for constant {:?}", value))?;
                let discr = adt_def.discriminant_for_variant(tcx, variant).val;
                let discr = if discr_ty.is_signed() {
                    let shift = 128 - discr_ty.bits();
                    ((discr as i128) << shift) >> shift
                } else {
                    discr as i128
                };
                return Ok(ForgeConst::Int(discr, discr_ty));
            }
            _ => {
                let fields = destructured
                    .fields
                    .iter()
                    .map(|(field_value, field_ty)| {
                        translate_const_value(tcx, *field_value, *field_ty)
                    })
                    .collect::<TranslateResult<Vec<_>>>()?;
                return Ok(ForgeConst::Aggregate(fields));
            }
        }
    }

    Err(format!(
        "unsupported constant value for type {:?}: {:?}",
        ty, value
    ))
}

fn scalar_backed_aggregate_field_ty<'tcx>(
    tcx: TyCtxt<'tcx>,
    ty: ty::Ty<'tcx>,
) -> Option<ty::Ty<'tcx>> {
    match ty.kind() {
        ty::TyKind::Tuple(fields) if fields.len() == 1 => Some(fields[0]),
        ty::TyKind::Adt(adt_def, args) if !adt_def.is_enum() => {
            let mut fields = adt_def.all_fields();
            let field = fields.next()?;
            if fields.next().is_some() {
                return None;
            }
            Some(field.ty(tcx, *args).skip_norm_wip())
        }
        _ => None,
    }
}

fn translate_ty_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    value: ty::Const<'tcx>,
    ty: ty::Ty<'tcx>,
) -> TranslateResult<ForgeConst> {
    if let Some(value) = value.try_to_value() {
        return translate_value_const(tcx, value);
    }

    let scalar = value
        .try_to_leaf()
        .ok_or_else(|| format!("unsupported type-system constant: {:?}", value))?;
    if let Some(field_ty) = scalar_backed_aggregate_field_ty(tcx, ty) {
        return Ok(ForgeConst::Aggregate(vec![translate_scalar_const(
            tcx, field_ty, scalar,
        )?]));
    }
    translate_scalar_const(tcx, ty, scalar)
}

fn translate_value_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    value: ty::Value<'tcx>,
) -> TranslateResult<ForgeConst> {
    match value.ty.kind() {
        ty::TyKind::Bool | ty::TyKind::Char | ty::TyKind::Int(_) | ty::TyKind::Uint(_) => {
            let scalar = value
                .try_to_leaf()
                .ok_or_else(|| format!("expected scalar constant, got {:?}", value))?;
            translate_scalar_const(tcx, value.ty, scalar)
        }
        ty::TyKind::Tuple(fields) => {
            let branches = value
                .try_to_branch()
                .ok_or_else(|| format!("expected tuple constant branches, got {:?}", value))?;
            let fields = branches
                .iter()
                .zip(fields.iter())
                .map(|(field_const, field_ty)| translate_ty_const(tcx, *field_const, field_ty))
                .collect::<TranslateResult<Vec<_>>>()?;
            Ok(ForgeConst::Aggregate(fields))
        }
        ty::TyKind::Array(inner, len) => {
            let branches = value
                .try_to_branch()
                .ok_or_else(|| format!("expected array constant branches, got {:?}", value))?;
            let len = len
                .try_to_target_usize(tcx)
                .ok_or_else(|| format!("unsupported array constant length: {:?}", len))?
                as usize;
            if branches.len() != len {
                return Err(format!(
                    "array constant branch count mismatch: expected {len}, got {}",
                    branches.len()
                ));
            }
            let fields = branches
                .iter()
                .map(|field_const| translate_ty_const(tcx, *field_const, *inner))
                .collect::<TranslateResult<Vec<_>>>()?;
            Ok(ForgeConst::Aggregate(fields))
        }
        ty::TyKind::Adt(adt_def, args) if adt_def.is_enum() => {
            if adt_def
                .variants()
                .iter()
                .any(|variant| !variant.fields.is_empty())
            {
                return Err(format!(
                    "unsupported enum constant with fields: {:?}",
                    value
                ));
            }
            let discr_ty = translate_ty_with_tcx(tcx, value.ty)
                .ok_or_else(|| format!("unsupported enum constant type: {:?}", value.ty))?;
            let variant = value.destructure_adt_const().variant;
            let discr = adt_def.discriminant_for_variant(tcx, variant).val;
            let discr = if discr_ty.is_signed() {
                let shift = 128 - discr_ty.bits();
                ((discr as i128) << shift) >> shift
            } else {
                discr as i128
            };
            Ok(ForgeConst::Int(discr, discr_ty))
        }
        ty::TyKind::Adt(adt_def, _) if adt_def.is_union() => {
            let scalar = value
                .try_to_leaf()
                .ok_or_else(|| format!("unsupported union constant: {value:?}"))?;
            translate_scalar_const(tcx, value.ty, scalar)
        }
        ty::TyKind::Adt(adt_def, args) => {
            let destructured = value.destructure_adt_const();
            let variant = &adt_def.variant(destructured.variant);
            let fields = destructured
                .fields
                .iter()
                .zip(variant.fields.iter())
                .map(|(field_const, field_def)| {
                    translate_ty_const(tcx, *field_const, field_def.ty(tcx, *args).skip_norm_wip())
                })
                .collect::<TranslateResult<Vec<_>>>()?;
            Ok(ForgeConst::Aggregate(fields))
        }
        _ => Err(format!("unsupported value constant: {:?}", value)),
    }
}

fn translate_scalar_const<'tcx>(
    tcx: TyCtxt<'tcx>,
    ty: ty::Ty<'tcx>,
    scalar: rustc_middle::ty::ScalarInt,
) -> TranslateResult<ForgeConst> {
    let forge_ty = translate_ty_with_tcx(tcx, ty)
        .ok_or_else(|| format!("unsupported scalar constant type: {:?}", ty))?;
    let bits = scalar.to_bits(scalar.size());

    let value = if forge_ty.is_signed() {
        let bit_width = forge_ty.bits();
        crate::ir::ty::sign_extend(bits, bit_width)
    } else {
        bits as i128
    };
    Ok(ForgeConst::Int(value, forge_ty))
}

fn operand_ty<'tcx>(
    tcx: TyCtxt<'tcx>,
    operand: &mir::Operand<'tcx>,
    body: &mir::Body<'tcx>,
) -> TranslateResult<ForgeTy> {
    match operand {
        mir::Operand::Copy(place) | mir::Operand::Move(place) => place_ty(tcx, place, body),
        mir::Operand::RuntimeChecks(_) => Err("unsupported RuntimeChecks operand type".into()),
        mir::Operand::Constant(constant) => translate_ty_with_tcx(tcx, constant.ty())
            .ok_or_else(|| format!("unsupported constant operand type: {:?}", constant.ty())),
    }
}

fn place_ty<'tcx>(
    tcx: TyCtxt<'tcx>,
    place: &mir::Place<'tcx>,
    body: &mir::Body<'tcx>,
) -> TranslateResult<ForgeTy> {
    let mut ty = body.local_decls[place.local].ty;
    for projection in place.projection.iter() {
        match projection {
            mir::ProjectionElem::Field(_, field_ty) => ty = field_ty,
            mir::ProjectionElem::ConstantIndex { .. } => {
                let (element_ty, _) = array_projection_info(tcx, ty)?;
                ty = element_ty;
            }
            mir::ProjectionElem::Index(_) => {
                let (element_ty, _) = array_projection_info(tcx, ty)?;
                ty = element_ty;
            }
            mir::ProjectionElem::Deref => {
                let ty::TyKind::Ref(_, inner, _) = ty.kind() else {
                    return Err(format!(
                        "unsupported deref projection while determining operand type: {:?}",
                        place
                    ));
                };
                ty = *inner;
            }
            _ => {
                return Err(format!(
                    "unsupported place projection while determining operand type: {:?}",
                    place
                ));
            }
        }
    }

    translate_ty_with_tcx(tcx, ty).ok_or_else(|| {
        format!(
            "unsupported local type for operand {:?}: {:?}",
            place.local, ty
        )
    })
}

fn array_projection_info<'tcx>(
    tcx: TyCtxt<'tcx>,
    ty: ty::Ty<'tcx>,
) -> TranslateResult<(ty::Ty<'tcx>, usize)> {
    let ty::TyKind::Array(element_ty, len) = ty.kind() else {
        return Err(format!("array projection on non-array type: {:?}", ty));
    };
    let len = len
        .try_to_target_usize(tcx)
        .ok_or_else(|| format!("unsupported array length: {:?}", len))?
        .try_into()
        .map_err(|_| format!("array length does not fit usize: {:?}", len))?;
    Ok((*element_ty, len))
}

fn translate_constant_index(
    offset: u64,
    min_length: u64,
    from_end: bool,
    len: usize,
) -> TranslateResult<usize> {
    if from_end {
        return Err(format!(
            "unsupported ConstantIndex from_end=true offset={offset} min_length={min_length}"
        ));
    }

    if len < min_length as usize {
        return Err(format!(
            "array length {len} smaller than required minimum length {min_length}"
        ));
    }

    let index = offset as usize;
    if index >= len {
        return Err(format!(
            "array constant index {index} out of bounds for length {len}"
        ));
    }

    Ok(index)
}

fn translate_rvalue<'tcx>(
    tcx: TyCtxt<'tcx>,
    rvalue: &mir::Rvalue<'tcx>,
    body: &mir::Body<'tcx>,
) -> TranslateResult<ForgeRvalue> {
    match rvalue {
        mir::Rvalue::Use(operand, _) => {
            Ok(ForgeRvalue::Use(translate_operand(tcx, operand, body)?))
        }
        mir::Rvalue::Ref(_, borrow_kind, place) => {
            if !matches!(borrow_kind, mir::BorrowKind::Mut { .. }) {
                return Err(format!("unsupported borrow kind in rvalue: {:?}", rvalue));
            }
            Ok(ForgeRvalue::Ref(translate_place(tcx, place, body)?))
        }
        mir::Rvalue::BinaryOp(bin_op, operands) => {
            let (lhs, rhs) = &**operands;
            let l = translate_operand(tcx, lhs, body)?;
            let r = translate_operand(tcx, rhs, body)?;
            let op = translate_binop(*bin_op)?;
            let ty = operand_ty(tcx, lhs, body)?;
            Ok(ForgeRvalue::BinaryOp {
                op,
                lhs: l,
                rhs: r,
                ty,
            })
        }
        mir::Rvalue::Discriminant(place) => Ok(ForgeRvalue::Use(ForgeOperand::Place(
            translate_place(tcx, place, body)?,
        ))),
        mir::Rvalue::UnaryOp(un_op, operand) => {
            let ty = operand_ty(tcx, operand, body)?;
            let op = match un_op {
                mir::UnOp::Neg => ForgeUnOp::Neg,
                mir::UnOp::Not => ForgeUnOp::Not,
                mir::UnOp::PtrMetadata => {
                    return Err(format!(
                        "unsupported unary op PtrMetadata in rvalue: {:?}",
                        rvalue
                    ));
                }
            };
            Ok(ForgeRvalue::UnaryOp {
                op,
                operand: translate_operand(tcx, operand, body)?,
                ty,
            })
        }
        mir::Rvalue::Cast(cast_kind, operand, ty) => {
            let src_ty = operand_ty(tcx, operand, body)?;
            let dst_ty = translate_ty_with_tcx(tcx, *ty)
                .ok_or_else(|| format!("unsupported cast destination type: {:?}", ty))?;
            match cast_kind {
                mir::CastKind::IntToInt => Ok(ForgeRvalue::Cast {
                    kind: ForgeCastKind::IntToInt,
                    operand: translate_operand(tcx, operand, body)?,
                    src_ty,
                    dst_ty,
                }),
                mir::CastKind::Transmute => {
                    if contains_unsupported_transmute_ty(&src_ty)
                        || contains_unsupported_transmute_ty(&dst_ty)
                    {
                        return Err(format!(
                            "unsupported transmute types: {:?} -> {:?}",
                            src_ty, dst_ty
                        ));
                    }
                    if src_ty.total_bits() != dst_ty.total_bits() {
                        return Err(format!(
                            "unsupported transmute with different bit widths: {:?} -> {:?}",
                            src_ty, dst_ty
                        ));
                    }
                    let src_layout = transmute_layout(tcx, operand.ty(body, tcx), &src_ty)?;
                    let dst_layout = transmute_layout(tcx, *ty, &dst_ty)?;
                    if src_layout.bits == 0 || src_layout.bits != dst_layout.bits {
                        return Err("unsupported transmute storage size".into());
                    }
                    Ok(ForgeRvalue::Cast {
                        kind: ForgeCastKind::Transmute {
                            little_endian: tcx.data_layout.endian.as_str() == "little",
                            src_layout,
                            dst_layout,
                        },
                        operand: translate_operand(tcx, operand, body)?,
                        src_ty,
                        dst_ty,
                    })
                }
                _ => Err(format!("unsupported cast kind: {:?}", cast_kind)),
            }
        }
        mir::Rvalue::Aggregate(kind, operands) => {
            if let mir::AggregateKind::Adt(def_id, _, _, _, Some(_)) = &**kind
                && tcx.adt_def(*def_id).is_union()
            {
                if operands.len() != 1 {
                    return Err("union construction must have one active field".into());
                }
                return Ok(ForgeRvalue::Use(translate_operand(
                    tcx,
                    operands.iter().next().unwrap(),
                    body,
                )?));
            }
            let fields = operands
                .iter()
                .map(|operand| translate_operand(tcx, operand, body))
                .collect::<TranslateResult<Vec<_>>>()?;
            Ok(ForgeRvalue::Aggregate(fields))
        }
        _ => Err(format!("unsupported rvalue: {:?}", rvalue)),
    }
}

fn translate_binop(op: mir::BinOp) -> TranslateResult<ForgeBinOp> {
    Ok(match op {
        mir::BinOp::Add => ForgeBinOp::Add,
        mir::BinOp::AddUnchecked => ForgeBinOp::AddUnchecked,
        mir::BinOp::AddWithOverflow => ForgeBinOp::AddWithOverflow,
        mir::BinOp::Sub => ForgeBinOp::Sub,
        mir::BinOp::SubUnchecked => ForgeBinOp::SubUnchecked,
        mir::BinOp::SubWithOverflow => ForgeBinOp::SubWithOverflow,
        mir::BinOp::Mul => ForgeBinOp::Mul,
        mir::BinOp::MulUnchecked => ForgeBinOp::MulUnchecked,
        mir::BinOp::MulWithOverflow => ForgeBinOp::MulWithOverflow,
        mir::BinOp::Shl | mir::BinOp::ShlUnchecked => ForgeBinOp::Shl,
        mir::BinOp::Shr | mir::BinOp::ShrUnchecked => ForgeBinOp::Shr,
        mir::BinOp::Div => ForgeBinOp::Div,
        mir::BinOp::Rem => ForgeBinOp::Rem,
        mir::BinOp::BitAnd => ForgeBinOp::BitAnd,
        mir::BinOp::BitOr => ForgeBinOp::BitOr,
        mir::BinOp::BitXor => ForgeBinOp::BitXor,
        mir::BinOp::Eq => ForgeBinOp::Eq,
        mir::BinOp::Ne => ForgeBinOp::Ne,
        mir::BinOp::Lt => ForgeBinOp::Lt,
        mir::BinOp::Le => ForgeBinOp::Le,
        mir::BinOp::Gt => ForgeBinOp::Gt,
        mir::BinOp::Ge => ForgeBinOp::Ge,
        _ => return Err(format!("unsupported binop: {:?}", op)),
    })
}

fn translate_terminator<'tcx>(
    tcx: TyCtxt<'tcx>,
    terminator: &mir::Terminator<'tcx>,
    body: &mir::Body<'tcx>,
) -> TranslateResult<ForgeTerminator> {
    match &terminator.kind {
        mir::TerminatorKind::Goto { target, .. } => {
            Ok(ForgeTerminator::Goto(BlockId(target.as_u32())))
        }
        mir::TerminatorKind::Assert {
            cond,
            expected,
            target,
            ..
        } => {
            let forge_cond = translate_operand(tcx, cond, body)?;
            let cond_ty = operand_ty(tcx, cond, body)?;
            let expected_ty = ForgeTy::Int(IntWidth::I8, IntKind::Bool);
            if cond_ty != expected_ty {
                return Err(format!("assert operand must be bool, got {:?}", cond_ty));
            }
            Ok(ForgeTerminator::Assert {
                cond: forge_cond,
                expected: *expected,
                target: BlockId(target.as_u32()),
            })
        }
        mir::TerminatorKind::SwitchInt { discr, targets } => {
            let forge_discr = translate_operand(tcx, discr, body)?;
            let mut forge_targets = Vec::new();
            for (value, target) in targets.iter() {
                forge_targets.push((value, BlockId(target.as_u32())));
            }
            let otherwise = BlockId(targets.otherwise().as_u32());
            Ok(ForgeTerminator::SwitchInt {
                discr: forge_discr,
                targets: forge_targets,
                otherwise,
            })
        }
        mir::TerminatorKind::Call {
            func,
            args,
            destination,
            target,
            unwind,
            ..
        } => Err(describe_unsupported_call(
            tcx,
            func,
            args,
            destination,
            *target,
            unwind,
            body,
        )),
        mir::TerminatorKind::Return => Ok(ForgeTerminator::Return),
        mir::TerminatorKind::Unreachable => Ok(ForgeTerminator::Unreachable),
        _ => Err(format!("unsupported terminator: {:?}", terminator.kind)),
    }
}

fn translate_call_terminator<'tcx>(
    tcx: TyCtxt<'tcx>,
    terminator: &mir::Terminator<'tcx>,
    body: &mir::Body<'tcx>,
) -> TranslateResult<Option<(ForgeStmt, ForgeTerminator)>> {
    let mir::TerminatorKind::Call {
        func,
        args,
        destination,
        target,
        ..
    } = &terminator.kind
    else {
        return Ok(None);
    };

    let Some(target) = target else {
        return Ok(None);
    };

    let Some(kind) = classify_intrinsic_call(tcx, func, body) else {
        return Ok(None);
    };

    match kind {
        IntrinsicCall::ByteSwap => {
            if args.len() != 1 {
                return Err(format!("bswap expects 1 argument, got {}", args.len()));
            }
            let operand = translate_operand(tcx, &args[0].node, body)?;
            let ty = operand_ty(tcx, &args[0].node, body)?;
            if !matches!(ty, ForgeTy::Int(_, _)) || !ty.bits().is_multiple_of(8) {
                return Err(format!(
                    "bswap requires an integer argument with byte width, got {:?}",
                    ty
                ));
            }
            let place = translate_place(tcx, destination, body)?;
            Ok(Some((
                ForgeStmt::Assign(place, ForgeRvalue::ByteSwap { operand, ty }),
                ForgeTerminator::Goto(BlockId(target.as_u32())),
            )))
        }
    }
}

enum IntrinsicCall {
    ByteSwap,
}

fn classify_intrinsic_call<'tcx>(
    tcx: TyCtxt<'tcx>,
    func: &mir::Operand<'tcx>,
    body: &mir::Body<'tcx>,
) -> Option<IntrinsicCall> {
    let ty = func.ty(body, tcx);
    let ty::TyKind::FnDef(def_id, _) = ty.kind() else {
        return None;
    };

    let name = tcx.item_name(*def_id);
    if name.as_str() == "bswap" && tcx.is_intrinsic(*def_id, name) {
        return Some(IntrinsicCall::ByteSwap);
    }
    let primitive_impl = tcx.impl_of_assoc(*def_id).is_some_and(|impl_id| {
        tcx.crate_name(def_id.krate).as_str() == "core"
            && matches!(
                tcx.type_of(impl_id)
                    .instantiate_identity()
                    .skip_norm_wip()
                    .kind(),
                ty::TyKind::Int(_) | ty::TyKind::Uint(_)
            )
    });
    if name.as_str() == "swap_bytes" && primitive_impl {
        return Some(IntrinsicCall::ByteSwap);
    }
    None
}

fn transmute_layout<'tcx>(
    tcx: TyCtxt<'tcx>,
    rust_ty: ty::Ty<'tcx>,
    forge_ty: &ForgeTy,
) -> TranslateResult<ForgeLayout> {
    let typing_env = ty::TypingEnv::fully_monomorphized();
    let layout = tcx
        .layout_of(typing_env.as_query_input(rust_ty))
        .map_err(|err| format!("cannot compute transmute layout for {rust_ty:?}: {err:?}"))?;
    let bits = u32::try_from(layout.size.bits()).map_err(|_| "transmute layout too large")?;
    let mut fields = Vec::new();
    match forge_ty {
        ForgeTy::Int(_, _) if bits == forge_ty.bits() => {}
        ForgeTy::Aggregate(field_tys) => {
            let rust_fields: Vec<_> = match rust_ty.kind() {
                ty::TyKind::Tuple(fields) => fields.iter().collect(),
                ty::TyKind::Array(element, _) => vec![*element; field_tys.len()],
                ty::TyKind::Adt(adt, args) if !adt.is_enum() && !adt.is_union() => adt
                    .all_fields()
                    .map(|field| field.ty(tcx, *args).skip_norm_wip())
                    .collect(),
                _ => {
                    return Err(format!(
                        "unsupported aggregate transmute layout: {rust_ty:?}"
                    ));
                }
            };
            for (index, (field_ty, rust_field)) in field_tys.iter().zip(rust_fields).enumerate() {
                let offset = u32::try_from(layout.fields.offset(index).bits())
                    .map_err(|_| "field offset too large")?;
                fields.push((offset, transmute_layout(tcx, rust_field, field_ty)?));
            }
            // Padding has no symbolic initialized bytes in Forge's value model.
            // Require complete coverage rather than inventing bits for it.
            let mut intervals: Vec<_> = fields
                .iter()
                .filter(|(_, field)| field.bits != 0)
                .map(|(offset, field)| (*offset, field.bits))
                .collect();
            intervals.sort_unstable();
            let mut end = 0;
            for (offset, width) in intervals {
                if offset != end {
                    return Err(format!(
                        "unsupported padding in transmute layout: {rust_ty:?}"
                    ));
                }
                end += width;
            }
            if end != bits {
                return Err(format!(
                    "unsupported padding in transmute layout: {rust_ty:?}"
                ));
            }
        }
        _ => return Err(format!("unsupported transmute layout: {rust_ty:?}")),
    }
    Ok(ForgeLayout { bits, fields })
}

fn contains_unsupported_transmute_ty(ty: &ForgeTy) -> bool {
    match ty {
        ForgeTy::Ref(_) => true,
        ForgeTy::Aggregate(fields) => fields.iter().any(contains_unsupported_transmute_ty),
        ForgeTy::Int(_, _) => false,
    }
}

fn describe_unsupported_call<'tcx>(
    tcx: TyCtxt<'tcx>,
    func: &mir::Operand<'tcx>,
    args: &[rustc_span::Spanned<mir::Operand<'tcx>>],
    destination: &mir::Place<'tcx>,
    target: Option<mir::BasicBlock>,
    unwind: &mir::UnwindAction,
    body: &mir::Body<'tcx>,
) -> String {
    let func_ty = match operand_ty(tcx, func, body) {
        Ok(ty) => format!("{ty:?}"),
        Err(err) => format!("<unknown: {err}>"),
    };
    let callee = describe_callee(tcx, func, body);
    let args = args
        .iter()
        .map(|arg| format!("{:?}", arg.node))
        .collect::<Vec<_>>()
        .join(", ");
    let target = target.map(|bb| format!("bb{}", bb.as_u32()));
    format!(
        "unsupported call terminator: callee={callee}, func={func:?}, func_ty={func_ty}, args=[{args}], destination={destination:?}, target={target:?}, unwind={unwind:?}",
    )
}

fn describe_callee<'tcx>(
    tcx: TyCtxt<'tcx>,
    func: &mir::Operand<'tcx>,
    body: &mir::Body<'tcx>,
) -> String {
    let ty = func.ty(body, tcx);
    let ty::TyKind::FnDef(def_id, _) = ty.kind() else {
        return String::from("<non-fn-def>");
    };
    let name = tcx.def_path_str(*def_id);
    let symbol = tcx.item_name(*def_id);
    if tcx.is_intrinsic(*def_id, symbol) {
        format!("intrinsic {name}")
    } else {
        name
    }
}

pub fn translate_body<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
) -> TranslateResult<ForgeBody> {
    let mut locals = Vec::new();
    for (local_idx, local_decl) in body.local_decls.iter_enumerated() {
        let forge_ty = translate_ty_with_tcx(tcx, local_decl.ty).ok_or_else(|| {
            format!(
                "unsupported local type for {:?}: {:?}",
                local_idx, local_decl.ty
            )
        })?;
        locals.push((VarId(locals.len() as u32), forge_ty));
    }

    let param_count = body.arg_count;

    let mut blocks = Vec::new();
    for (bb_idx, bb_data) in body.basic_blocks.iter_enumerated() {
        let mut stmts = Vec::new();
        for stmt in &bb_data.statements {
            match &stmt.kind {
                mir::StatementKind::Assign(assignment) => {
                    let (place, rvalue) = &**assignment;
                    let forge_place = translate_place(tcx, place, body).map_err(|err| {
                        format!("bb{} statement {:?}: {}", bb_idx.as_u32(), stmt.kind, err)
                    })?;
                    let forge_rvalue = translate_rvalue(tcx, rvalue, body).map_err(|err| {
                        format!("bb{} statement {:?}: {}", bb_idx.as_u32(), stmt.kind, err)
                    })?;
                    stmts.push(ForgeStmt::Assign(forge_place, forge_rvalue));
                }
                mir::StatementKind::Intrinsic(intrinsic) => {
                    let mir::NonDivergingIntrinsic::Assume(operand) = &**intrinsic else {
                        return Err(format!(
                            "bb{} unsupported statement: {:?}",
                            bb_idx.as_u32(),
                            stmt.kind
                        ));
                    };
                    let forge_operand = translate_operand(tcx, operand, body).map_err(|err| {
                        format!("bb{} statement {:?}: {}", bb_idx.as_u32(), stmt.kind, err)
                    })?;
                    let operand_ty = operand_ty(tcx, operand, body).map_err(|err| {
                        format!("bb{} statement {:?}: {}", bb_idx.as_u32(), stmt.kind, err)
                    })?;
                    let expected = ForgeTy::Int(IntWidth::I8, IntKind::Bool);
                    if operand_ty != expected {
                        return Err(format!(
                            "bb{} statement {:?}: assume operand must be bool, got {:?}",
                            bb_idx.as_u32(),
                            stmt.kind,
                            operand_ty
                        ));
                    }
                    stmts.push(ForgeStmt::Assume(ForgeAssume::Bool(forge_operand)));
                }
                mir::StatementKind::StorageLive(_)
                | mir::StatementKind::StorageDead(_)
                | mir::StatementKind::Nop => {}
                _ => {
                    return Err(format!(
                        "bb{} unsupported statement: {:?}",
                        bb_idx.as_u32(),
                        stmt.kind
                    ));
                }
            }
        }

        let terminator = bb_data.terminator();
        if let Some((stmt, forge_terminator)) = translate_call_terminator(tcx, terminator, body)
            .map_err(|err| {
                format!(
                    "bb{} terminator {:?}: {}",
                    bb_idx.as_u32(),
                    terminator.kind,
                    err
                )
            })?
        {
            stmts.push(stmt);
            blocks.push(ForgeBlock {
                id: BlockId(bb_idx.as_u32()),
                stmts,
                terminator: forge_terminator,
            });
            continue;
        }
        let forge_terminator = translate_terminator(tcx, terminator, body).map_err(|err| {
            format!(
                "bb{} terminator {:?}: {}",
                bb_idx.as_u32(),
                terminator.kind,
                err
            )
        })?;

        blocks.push(ForgeBlock {
            id: BlockId(bb_idx.as_u32()),
            stmts,
            terminator: forge_terminator,
        });
    }

    Ok(ForgeBody {
        locals,
        param_count,
        blocks,
    })
}
