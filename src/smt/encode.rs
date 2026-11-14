use std::collections::HashMap;

use z3::ast::{self, BV};

use crate::ir::inst::*;
use crate::ir::ty::{ForgeLayout, ForgeTy};

#[derive(Clone)]
pub enum SymValue {
    Scalar(BV),
    Aggregate(Vec<SymValue>),
    Ref(SymPlace),
    UninitRef,
}

/// A resolved address. Dynamic indices capture expressions at borrow time.
#[derive(Clone, PartialEq, Eq)]
pub struct SymPlace {
    pub base: VarId,
    projections: Vec<SymProjection>,
}

#[derive(Clone, PartialEq, Eq)]
enum SymProjection {
    Field(u32),
    Index(BV),
}

impl SymPlace {
    pub fn root(base: VarId) -> Self {
        Self {
            base,
            projections: Vec::new(),
        }
    }
}

/// Result of symbolic execution: a list of (path_condition, return_value) pairs.
pub struct SymExecResult {
    pub returns: Vec<(ast::Bool, SymValue)>,
    pub final_states: Vec<(ast::Bool, HashMap<VarId, SymValue>)>,
    pub ub_paths: ast::Bool,
    pub unwind_paths: ast::Bool,
}

struct EvalResult {
    value: SymValue,
    ub: ast::Bool,
}

struct SymState {
    vars: HashMap<VarId, SymValue>,
}

impl SymState {
    fn eval_place(&self, place: &ForgePlace) -> SymValue {
        self.eval_place_value(place)
    }

    fn eval_place_value(&self, place: &ForgePlace) -> SymValue {
        self.eval_resolved_place(&self.resolve_place(place))
    }

    fn eval_resolved_place(&self, place: &SymPlace) -> SymValue {
        let mut current = self.vars[&place.base].clone();
        for projection in &place.projections {
            current = match projection {
                SymProjection::Field(field) => extract_field(current, *field),
                SymProjection::Index(index) => extract_index(current, index),
            };
        }
        current
    }

    fn resolve_place(&self, place: &ForgePlace) -> SymPlace {
        let mut resolved = SymPlace {
            base: place.base,
            projections: Vec::new(),
        };
        let mut current = self.vars[&place.base].clone();

        for projection in &place.projections {
            match projection {
                ForgeProjection::Field(field) => {
                    resolved.projections.push(SymProjection::Field(*field));
                    current = extract_field(current, *field);
                }
                ForgeProjection::Index(var) => {
                    let index = self.vars[var].clone().into_scalar();
                    resolved
                        .projections
                        .push(SymProjection::Index(index.clone()));
                    current = extract_index(current, &index);
                }
                ForgeProjection::Deref => {
                    let SymValue::Ref(target) = current else {
                        panic!("deref from non-reference value");
                    };
                    resolved = target;
                    current = self.eval_resolved_place(&resolved);
                }
            }
        }

        resolved
    }

    fn eval_operand(&self, op: &ForgeOperand) -> SymValue {
        match op {
            ForgeOperand::Place(place) => self.eval_place(place),
            ForgeOperand::Const(ForgeConst::Int(value, ty)) => eval_const_int(*value, ty),
            ForgeOperand::Const(ForgeConst::Aggregate(fields)) => SymValue::Aggregate(
                fields
                    .iter()
                    .cloned()
                    .map(|field| self.eval_operand(&ForgeOperand::Const(field)))
                    .collect(),
            ),
        }
    }

    fn eval_scalar_operand(&self, op: &ForgeOperand) -> BV {
        self.eval_operand(op).into_scalar()
    }

    fn eval_rvalue(&self, rv: &ForgeRvalue) -> EvalResult {
        match rv {
            ForgeRvalue::Use(op) => EvalResult {
                value: self.eval_operand(op),
                ub: ast::Bool::from_bool(false),
            },
            ForgeRvalue::Ref(place) => EvalResult {
                value: SymValue::Ref(self.resolve_place(place)),
                ub: ast::Bool::from_bool(false),
            },
            ForgeRvalue::ByteSwap { operand, ty } => {
                let value = self.eval_scalar_operand(operand);
                EvalResult {
                    value: SymValue::Scalar(eval_bswap(&value, ty)),
                    ub: ast::Bool::from_bool(false),
                }
            }
            ForgeRvalue::BinaryOp { op, lhs, rhs, ty } => {
                let l = self.eval_scalar_operand(lhs);
                let r = self.eval_scalar_operand(rhs);
                let eval = eval_binop(*op, ty, &l, &r);
                EvalResult {
                    value: eval.value,
                    ub: eval.ub,
                }
            }
            ForgeRvalue::UnaryOp { op, operand, ty } => {
                let operand = self.eval_operand(operand);
                let value = match (op, operand) {
                    (ForgeUnOp::Neg, SymValue::Scalar(v)) => SymValue::Scalar(v.bvneg()),
                    (ForgeUnOp::Not, SymValue::Scalar(v)) => {
                        if ty.is_bool() {
                            SymValue::Scalar(v.bvxor(BV::from_u64(1, v.get_size())))
                        } else {
                            SymValue::Scalar(v.bvnot())
                        }
                    }
                    (_, SymValue::Aggregate(_) | SymValue::Ref(_) | SymValue::UninitRef) => {
                        panic!("invalid unary op operand")
                    }
                };
                EvalResult {
                    value,
                    ub: ast::Bool::from_bool(false),
                }
            }
            ForgeRvalue::Cast {
                kind: ForgeCastKind::IntToInt,
                operand,
                src_ty,
                dst_ty,
            } => {
                let v = self.eval_scalar_operand(operand);
                let src_bits = src_ty.bits();
                let dst_bits = dst_ty.bits();
                let value = if dst_bits > src_bits {
                    if src_ty.is_signed() {
                        v.sign_ext(dst_bits - src_bits)
                    } else {
                        v.zero_ext(dst_bits - src_bits)
                    }
                } else if dst_bits < src_bits {
                    v.extract(dst_bits - 1, 0)
                } else {
                    v
                };
                EvalResult {
                    value: SymValue::Scalar(value),
                    ub: ast::Bool::from_bool(false),
                }
            }
            ForgeRvalue::Cast {
                kind:
                    ForgeCastKind::Transmute {
                        little_endian,
                        src_layout,
                        dst_layout,
                    },
                operand,
                src_ty,
                dst_ty,
            } => {
                let value = self.eval_operand(operand);
                let bits = pack_value_bits(value, src_ty, src_layout, *little_endian);
                EvalResult {
                    value: unpack_value_bits(&bits, dst_ty, dst_layout, *little_endian),
                    ub: ast::Bool::from_bool(false),
                }
            }
            ForgeRvalue::Aggregate(fields) => EvalResult {
                value: SymValue::Aggregate(
                    fields
                        .iter()
                        .map(|field| self.eval_operand(field))
                        .collect(),
                ),
                ub: ast::Bool::from_bool(false),
            },
        }
    }

    fn eval_assume(&self, assume: &ForgeAssume) -> ast::Bool {
        match assume {
            ForgeAssume::Bool(operand) => {
                let value = self.eval_scalar_operand(operand);
                value.eq(BV::from_u64(1, value.get_size()))
            }
            ForgeAssume::OneOf { operand, allowed } => {
                let value = self.eval_scalar_operand(operand);
                let conds: Vec<ast::Bool> = allowed
                    .iter()
                    .map(|allowed| match allowed {
                        ForgeConst::Int(int, ty) => value.eq(bv_from_int(*int, ty)),
                        ForgeConst::Aggregate(_) => {
                            panic!("aggregate values are not supported in assume one-of")
                        }
                    })
                    .collect();
                let refs: Vec<&ast::Bool> = conds.iter().collect();
                ast::Bool::or(&refs)
            }
        }
    }

    fn assign_place(&mut self, place: &ForgePlace, value: SymValue, path: &ast::Bool) {
        let resolved = self.resolve_place(place);
        let current = self.vars[&resolved.base].clone();
        let updated = self.update_projection(current, &resolved.projections, value, path);
        self.vars.insert(resolved.base, updated);
    }

    fn update_projection(
        &self,
        current: SymValue,
        projections: &[SymProjection],
        value: SymValue,
        path: &ast::Bool,
    ) -> SymValue {
        if projections.is_empty() {
            if matches!(value, SymValue::Ref(_)) && !path.as_bool().unwrap_or(false) {
                unimplemented!("conditional merging of reference values is not supported");
            }
            return merge_values(path, value, current);
        }

        let SymValue::Aggregate(mut fields) = current else {
            panic!("projection assignment to scalar value");
        };

        match &projections[0] {
            SymProjection::Field(idx) => {
                let idx = *idx as usize;
                let field = fields[idx].clone();
                fields[idx] = self.update_projection(field, &projections[1..], value, path);
            }
            SymProjection::Index(index) => {
                for (idx, field) in fields.clone().into_iter().enumerate() {
                    let idx_cond = index.eq(BV::from_u64(idx as u64, index.get_size()));
                    let branch_path = ast::Bool::and(&[path, &idx_cond]);
                    fields[idx] = self.update_projection(
                        field,
                        &projections[1..],
                        value.clone(),
                        &branch_path,
                    );
                }
            }
        }

        SymValue::Aggregate(fields)
    }
}

impl SymValue {
    pub fn ty(&self) -> ForgeTy {
        match self {
            SymValue::Scalar(bv) => ForgeTy::Int(
                match bv.get_size() {
                    1 => crate::ir::ty::IntWidth::I1,
                    8 => crate::ir::ty::IntWidth::I8,
                    16 => crate::ir::ty::IntWidth::I16,
                    32 => crate::ir::ty::IntWidth::I32,
                    64 => crate::ir::ty::IntWidth::I64,
                    128 => crate::ir::ty::IntWidth::I128,
                    bits => panic!("unsupported scalar width: {bits}"),
                },
                crate::ir::ty::IntKind::Unsigned,
            ),
            SymValue::Aggregate(fields) => {
                ForgeTy::Aggregate(fields.iter().map(SymValue::ty).collect())
            }
            SymValue::Ref(_) => panic!("reference values do not have a self-describing type"),
            SymValue::UninitRef => panic!("uninitialized reference"),
        }
    }

    pub fn into_scalar(self) -> BV {
        match self {
            SymValue::Scalar(bv) => bv,
            SymValue::Aggregate(_) | SymValue::Ref(_) | SymValue::UninitRef => {
                panic!("expected scalar value")
            }
        }
    }
}

fn extract_field(value: SymValue, field: u32) -> SymValue {
    let SymValue::Aggregate(fields) = value else {
        panic!("field projection from non-aggregate value");
    };
    fields[field as usize].clone()
}

fn extract_index(value: SymValue, index: &BV) -> SymValue {
    let SymValue::Aggregate(fields) = value else {
        panic!("index projection from non-aggregate value");
    };

    let mut selected = fields
        .first()
        .cloned()
        .expect("index projection from empty aggregate");

    for (idx, field) in fields.into_iter().enumerate().skip(1) {
        let cond = index.eq(BV::from_u64(idx as u64, index.get_size()));
        selected = merge_values(&cond, field, selected);
    }

    selected
}

pub fn merge_values(cond: &ast::Bool, then_value: SymValue, else_value: SymValue) -> SymValue {
    if let Some(value) = cond.as_bool() {
        return if value { then_value } else { else_value };
    }

    match (then_value, else_value) {
        (SymValue::Scalar(then_bv), SymValue::Scalar(else_bv)) => {
            SymValue::Scalar(cond.ite(&then_bv, &else_bv))
        }
        (SymValue::Aggregate(then_fields), SymValue::Aggregate(else_fields)) => {
            SymValue::Aggregate(
                then_fields
                    .into_iter()
                    .zip(else_fields)
                    .map(|(then_field, else_field)| merge_values(cond, then_field, else_field))
                    .collect(),
            )
        }
        (SymValue::Ref(then_place), SymValue::Ref(else_place)) => {
            if then_place == else_place {
                SymValue::Ref(then_place)
            } else {
                unimplemented!("merging distinct reference values is not supported");
            }
        }
        (SymValue::UninitRef, SymValue::UninitRef) => SymValue::UninitRef,
        _ => panic!("mismatched value shapes in merge"),
    }
}

pub fn values_equal(lhs: &SymValue, rhs: &SymValue) -> ast::Bool {
    match (lhs, rhs) {
        (SymValue::Scalar(lhs), SymValue::Scalar(rhs)) => lhs.eq(rhs),
        (SymValue::Aggregate(lhs), SymValue::Aggregate(rhs)) => {
            let conds = lhs
                .iter()
                .zip(rhs)
                .map(|(lhs, rhs)| values_equal(lhs, rhs))
                .collect::<Vec<_>>();
            let refs = conds.iter().collect::<Vec<_>>();
            ast::Bool::and(&refs)
        }
        (SymValue::Ref(_), _) | (_, SymValue::Ref(_)) => {
            unimplemented!("reference equality is not supported")
        }
        (SymValue::UninitRef, _) | (_, SymValue::UninitRef) => {
            panic!("comparison with uninitialized reference")
        }
        _ => panic!("mismatched value shapes in comparison"),
    }
}

struct BinOpEvalResult {
    value: SymValue,
    ub: ast::Bool,
}

enum OverflowMode {
    Plain,
    Unchecked,
    WithOverflow,
}

fn eval_binop(op: ForgeBinOp, ty: &ForgeTy, l: &BV, r: &BV) -> BinOpEvalResult {
    let false_ = ast::Bool::from_bool(false);
    let zero = BV::from_u64(0, r.get_size());
    let shift_amount = normalize_shift_amount(r, l.get_size());
    let (value, ub) = match op {
        ForgeBinOp::Add => eval_arith_binop(
            ty,
            l,
            r,
            OverflowMode::Plain,
            |l, r| l.bvadd(r),
            add_overflow,
        ),
        ForgeBinOp::AddUnchecked => eval_arith_binop(
            ty,
            l,
            r,
            OverflowMode::Unchecked,
            |l, r| l.bvadd(r),
            add_overflow,
        ),
        ForgeBinOp::AddWithOverflow => eval_arith_binop(
            ty,
            l,
            r,
            OverflowMode::WithOverflow,
            |l, r| l.bvadd(r),
            add_overflow,
        ),
        ForgeBinOp::Sub => eval_arith_binop(
            ty,
            l,
            r,
            OverflowMode::Plain,
            |l, r| l.bvsub(r),
            sub_overflow,
        ),
        ForgeBinOp::SubUnchecked => eval_arith_binop(
            ty,
            l,
            r,
            OverflowMode::Unchecked,
            |l, r| l.bvsub(r),
            sub_overflow,
        ),
        ForgeBinOp::SubWithOverflow => eval_arith_binop(
            ty,
            l,
            r,
            OverflowMode::WithOverflow,
            |l, r| l.bvsub(r),
            sub_overflow,
        ),
        ForgeBinOp::Mul => eval_arith_binop(
            ty,
            l,
            r,
            OverflowMode::Plain,
            |l, r| l.bvmul(r),
            mul_overflow,
        ),
        ForgeBinOp::MulUnchecked => eval_arith_binop(
            ty,
            l,
            r,
            OverflowMode::Unchecked,
            |l, r| l.bvmul(r),
            mul_overflow,
        ),
        ForgeBinOp::MulWithOverflow => eval_arith_binop(
            ty,
            l,
            r,
            OverflowMode::WithOverflow,
            |l, r| l.bvmul(r),
            mul_overflow,
        ),
        ForgeBinOp::Div => {
            let ub = r.eq(&zero);
            let value = if ty.is_signed() {
                l.bvsdiv(r)
            } else {
                l.bvudiv(r)
            };
            let overflow = signed_div_overflow(ty, l, r);
            (SymValue::Scalar(value), ast::Bool::or(&[&ub, &overflow]))
        }
        ForgeBinOp::Rem => {
            let ub = r.eq(&zero);
            let value = if ty.is_signed() {
                l.bvsrem(r)
            } else {
                l.bvurem(r)
            };
            let overflow = signed_div_overflow(ty, l, r);
            (SymValue::Scalar(value), ast::Bool::or(&[&ub, &overflow]))
        }
        ForgeBinOp::BitAnd => (SymValue::Scalar(l.bvand(r)), false_),
        ForgeBinOp::BitOr => (SymValue::Scalar(l.bvor(r)), false_),
        ForgeBinOp::BitXor => (SymValue::Scalar(l.bvxor(r)), false_),
        ForgeBinOp::Shl => (SymValue::Scalar(l.bvshl(&shift_amount)), false_),
        ForgeBinOp::Shr => {
            let value = if ty.is_signed() {
                l.bvashr(&shift_amount)
            } else {
                l.bvlshr(&shift_amount)
            };
            (SymValue::Scalar(value), false_)
        }
        ForgeBinOp::Eq => (SymValue::Scalar(bool_to_bv(&l.eq(r))), false_),
        ForgeBinOp::Ne => (SymValue::Scalar(bool_to_bv(&l.eq(r).not())), false_),
        ForgeBinOp::Lt => {
            let cmp = if ty.is_signed() {
                l.bvslt(r)
            } else {
                l.bvult(r)
            };
            (SymValue::Scalar(bool_to_bv(&cmp)), false_)
        }
        ForgeBinOp::Le => {
            let cmp = if ty.is_signed() {
                l.bvsle(r)
            } else {
                l.bvule(r)
            };
            (SymValue::Scalar(bool_to_bv(&cmp)), false_)
        }
        ForgeBinOp::Gt => {
            let cmp = if ty.is_signed() {
                l.bvsgt(r)
            } else {
                l.bvugt(r)
            };
            (SymValue::Scalar(bool_to_bv(&cmp)), false_)
        }
        ForgeBinOp::Ge => {
            let cmp = if ty.is_signed() {
                l.bvsge(r)
            } else {
                l.bvuge(r)
            };
            (SymValue::Scalar(bool_to_bv(&cmp)), false_)
        }
    };

    BinOpEvalResult { value, ub }
}

fn eval_bswap(value: &BV, ty: &ForgeTy) -> BV {
    let bits = ty.bits();
    assert!(
        bits.is_multiple_of(8),
        "bswap requires byte-sized integer type, got {:?}",
        ty
    );

    let byte_count = bits / 8;
    let mut result = value.extract(7, 0);
    for byte_index in 1..byte_count {
        let hi = byte_index * 8 + 7;
        let lo = byte_index * 8;
        result = result.concat(value.extract(hi, lo));
    }
    result
}

fn normalize_shift_amount(value: &BV, width: u32) -> BV {
    assert!(
        width.is_power_of_two(),
        "shift width must be a power of two"
    );
    let shift_bits = width.trailing_zeros();
    if shift_bits == 0 {
        return BV::from_u64(0, width);
    }

    let current = value.get_size();
    if current == shift_bits {
        value.clone().zero_ext(width - shift_bits)
    } else if current < shift_bits {
        value
            .zero_ext(shift_bits - current)
            .zero_ext(width - shift_bits)
    } else {
        value
            .extract(shift_bits - 1, 0)
            .zero_ext(width - shift_bits)
    }
}

fn pack_value_bits(value: SymValue, ty: &ForgeTy, layout: &ForgeLayout, little_endian: bool) -> BV {
    match (value, ty) {
        (SymValue::Scalar(bits), ForgeTy::Int(_, _)) => bits,
        (SymValue::Aggregate(fields), ForgeTy::Aggregate(field_tys)) => {
            assert_eq!(fields.len(), field_tys.len());
            assert_eq!(fields.len(), layout.fields.len());
            let mut packed = BV::from_u64(0, layout.bits);
            for ((field, field_ty), (offset, field_layout)) in
                fields.into_iter().zip(field_tys).zip(&layout.fields)
            {
                if field_layout.bits == 0 {
                    continue;
                }
                let bits = pack_value_bits(field, field_ty, field_layout, little_endian);
                let shift = if little_endian {
                    *offset
                } else {
                    layout.bits - offset - field_layout.bits
                };
                packed = packed.bvor(
                    bits.zero_ext(layout.bits - field_layout.bits)
                        .bvshl(BV::from_u64(shift as u64, layout.bits)),
                );
            }
            packed
        }
        _ => panic!("unsupported transmute value/type combination for {ty:?}"),
    }
}

fn unpack_value_bits(
    bits: &BV,
    ty: &ForgeTy,
    layout: &ForgeLayout,
    little_endian: bool,
) -> SymValue {
    match ty {
        ForgeTy::Int(_, _) => SymValue::Scalar(bits.clone()),
        ForgeTy::Aggregate(fields) => SymValue::Aggregate(
            fields
                .iter()
                .zip(&layout.fields)
                .map(|(field_ty, (offset, field_layout))| {
                    if field_layout.bits == 0 {
                        return zero_value(field_ty);
                    }
                    let shift = if little_endian {
                        *offset
                    } else {
                        layout.bits - offset - field_layout.bits
                    };
                    let field_bits = bits.extract(shift + field_layout.bits - 1, shift);
                    unpack_value_bits(&field_bits, field_ty, field_layout, little_endian)
                })
                .collect(),
        ),
        ForgeTy::Ref(_) => panic!("unsupported transmute destination type"),
    }
}

fn eval_arith_binop<F, O>(
    ty: &ForgeTy,
    l: &BV,
    r: &BV,
    mode: OverflowMode,
    compute: F,
    overflow_check: O,
) -> (SymValue, ast::Bool)
where
    F: Fn(&BV, &BV) -> BV,
    O: Fn(&ForgeTy, &BV, &BV, &BV) -> ast::Bool,
{
    let result = compute(l, r);
    let overflow = overflow_check(ty, l, r, &result);

    match mode {
        OverflowMode::Plain => (SymValue::Scalar(result), ast::Bool::from_bool(false)),
        OverflowMode::Unchecked => (SymValue::Scalar(result), overflow),
        OverflowMode::WithOverflow => (
            SymValue::Aggregate(vec![
                SymValue::Scalar(result),
                SymValue::Scalar(bool_to_bv(&overflow)),
            ]),
            ast::Bool::from_bool(false),
        ),
    }
}

fn add_overflow(ty: &ForgeTy, l: &BV, r: &BV, sum: &BV) -> ast::Bool {
    let ext_l = if ty.is_signed() {
        l.sign_ext(1)
    } else {
        l.zero_ext(1)
    };
    let ext_r = if ty.is_signed() {
        r.sign_ext(1)
    } else {
        r.zero_ext(1)
    };
    let ext_sum = ext_l.bvadd(&ext_r);
    let ext_trunc = if ty.is_signed() {
        sum.sign_ext(1)
    } else {
        sum.zero_ext(1)
    };
    ext_sum.eq(&ext_trunc).not()
}

fn sub_overflow(ty: &ForgeTy, l: &BV, r: &BV, diff: &BV) -> ast::Bool {
    let ext_l = if ty.is_signed() {
        l.sign_ext(1)
    } else {
        l.zero_ext(1)
    };
    let ext_r = if ty.is_signed() {
        r.sign_ext(1)
    } else {
        r.zero_ext(1)
    };
    let ext_diff = ext_l.bvsub(&ext_r);
    let ext_trunc = if ty.is_signed() {
        diff.sign_ext(1)
    } else {
        diff.zero_ext(1)
    };
    ext_diff.eq(&ext_trunc).not()
}

fn mul_overflow(ty: &ForgeTy, l: &BV, r: &BV, product: &BV) -> ast::Bool {
    let ext_l = if ty.is_signed() {
        l.sign_ext(product.get_size())
    } else {
        l.zero_ext(product.get_size())
    };
    let ext_r = if ty.is_signed() {
        r.sign_ext(product.get_size())
    } else {
        r.zero_ext(product.get_size())
    };
    let ext_product = ext_l.bvmul(&ext_r);
    let ext_trunc = if ty.is_signed() {
        product.sign_ext(product.get_size())
    } else {
        product.zero_ext(product.get_size())
    };
    ext_product.eq(&ext_trunc).not()
}

fn signed_div_overflow(ty: &ForgeTy, l: &BV, r: &BV) -> ast::Bool {
    if ty.is_signed() {
        let min = signed_min(ty);
        let neg_one = bv_all_ones(ty.bits());
        ast::Bool::and(&[&l.eq(&min), &r.eq(&neg_one)])
    } else {
        ast::Bool::from_bool(false)
    }
}

fn bool_to_bv(b: &ast::Bool) -> BV {
    b.ite(&BV::from_i64(1, 8), &BV::from_i64(0, 8))
}

fn bv_all_ones(bits: u32) -> BV {
    BV::from_u64(0, bits).bvnot()
}

fn bv_from_uint_raw(value: u128, bits: u32) -> BV {
    let masked = if bits == 128 {
        value
    } else {
        let mask = (1u128 << bits) - 1;
        value & mask
    };
    BV::from_str(bits, &masked.to_string()).unwrap()
}

fn signed_min(ty: &ForgeTy) -> BV {
    let bits = ty.bits();
    let unsigned = if bits == 128 {
        1u128 << 127
    } else {
        1u128 << (bits - 1)
    };
    BV::from_str(bits, &unsigned.to_string()).unwrap()
}

fn bv_from_int(value: i128, ty: &ForgeTy) -> BV {
    let bits = ty.bits();
    let unsigned = if bits == 128 {
        value as u128
    } else {
        let mask = (1u128 << bits) - 1;
        (value as u128) & mask
    };
    BV::from_str(bits, &unsigned.to_string()).unwrap()
}

fn eval_const_int(value: i128, ty: &ForgeTy) -> SymValue {
    match ty {
        ForgeTy::Int(_, _) => SymValue::Scalar(bv_from_int(value, ty)),
        ForgeTy::Aggregate(fields) if fields.len() == 1 => {
            SymValue::Aggregate(vec![eval_const_int(value, &fields[0])])
        }
        ForgeTy::Aggregate(_) => {
            panic!("integer constant with non-scalar aggregate type: {:?}", ty)
        }
        ForgeTy::Ref(_) => {
            panic!("integer constant with non-integer type: {:?}", ty)
        }
    }
}

fn zero_value(ty: &ForgeTy) -> SymValue {
    match ty {
        ForgeTy::Int(_, _) => SymValue::Scalar(BV::from_i64(0, ty.bits())),
        ForgeTy::Aggregate(fields) => SymValue::Aggregate(fields.iter().map(zero_value).collect()),
        ForgeTy::Ref(_) => SymValue::UninitRef,
    }
}

pub fn symbolic_value(ty: &ForgeTy, prefix: &str) -> SymValue {
    match ty {
        ForgeTy::Int(_, _) => SymValue::Scalar(BV::new_const(prefix, ty.bits())),
        ForgeTy::Aggregate(fields) => SymValue::Aggregate(
            fields
                .iter()
                .enumerate()
                .map(|(idx, field)| symbolic_value(field, &format!("{prefix}_{idx}")))
                .collect(),
        ),
        ForgeTy::Ref(_) => unimplemented!("reference parameters are not supported yet"),
    }
}

/// Symbolically execute a ForgeBody, producing (path_condition, return_value) pairs.
pub fn sym_exec(body: &ForgeBody, initial_vars: &HashMap<VarId, SymValue>) -> SymExecResult {
    let mut state = SymState {
        vars: HashMap::new(),
    };

    for (var_id, ty) in &body.locals {
        if let Some(pv) = initial_vars.get(var_id) {
            state.vars.insert(*var_id, pv.clone());
        } else {
            state.vars.insert(*var_id, zero_value(ty));
        }
    }
    for (var_id, value) in initial_vars {
        state.vars.entry(*var_id).or_insert_with(|| value.clone());
    }

    let num_blocks = body.blocks.len();
    let mut block_path: Vec<ast::Bool> = vec![ast::Bool::from_bool(false); num_blocks];
    block_path[0] = ast::Bool::from_bool(true);

    let mut returns: Vec<(ast::Bool, SymValue)> = Vec::new();
    let mut final_states: Vec<(ast::Bool, HashMap<VarId, SymValue>)> = Vec::new();
    let mut ub_paths = ast::Bool::from_bool(false);
    let mut unwind_paths = ast::Bool::from_bool(false);

    for block in &body.blocks {
        let bid = block.id.0 as usize;
        let path = block_path[bid].clone();

        for stmt in &block.stmts {
            match stmt {
                ForgeStmt::Assign(place, rvalue) => {
                    let eval = state.eval_rvalue(rvalue);
                    state.assign_place(place, eval.value, &path);
                    let ub_here = ast::Bool::and(&[&path, &eval.ub]);
                    ub_paths = ast::Bool::or(&[&ub_paths, &ub_here]);
                }
                ForgeStmt::Assume(assume) => {
                    let cond = state.eval_assume(assume);
                    let ub_here = ast::Bool::and(&[&path, &cond.not()]);
                    ub_paths = ast::Bool::or(&[&ub_paths, &ub_here]);
                }
            }
        }

        match &block.terminator {
            ForgeTerminator::Goto(target) => {
                let tid = target.0 as usize;
                block_path[tid] = ast::Bool::or(&[&block_path[tid], &path]);
            }
            ForgeTerminator::Assert {
                cond,
                expected,
                target,
            } => {
                let cond = state.eval_scalar_operand(cond);
                let expected = BV::from_u64(u64::from(*expected), cond.get_size());
                let success = cond.eq(expected);
                let success_path = ast::Bool::and(&[&path, &success]);
                let fail_path = ast::Bool::and(&[&path, &success.not()]);
                let tid = target.0 as usize;
                block_path[tid] = ast::Bool::or(&[&block_path[tid], &success_path]);
                unwind_paths = ast::Bool::or(&[&unwind_paths, &fail_path]);
            }
            ForgeTerminator::SwitchInt {
                discr,
                targets,
                otherwise,
            } => {
                let discr_val = state.eval_scalar_operand(discr);
                let mut covered = ast::Bool::from_bool(false);

                for (value, target) in targets {
                    let match_val = bv_from_uint_raw(*value, discr_val.get_size());
                    let cond = discr_val.eq(&match_val);
                    let branch_path = ast::Bool::and(&[&path, &cond]);
                    let tid = target.0 as usize;
                    block_path[tid] = ast::Bool::or(&[&block_path[tid], &branch_path]);
                    covered = ast::Bool::or(&[&covered, &cond]);
                }

                let otherwise_path = ast::Bool::and(&[&path, &covered.not()]);
                let oid = otherwise.0 as usize;
                block_path[oid] = ast::Bool::or(&[&block_path[oid], &otherwise_path]);
            }
            ForgeTerminator::Return => {
                let ret_val = state.vars[&VarId(0)].clone();
                returns.push((path.clone(), ret_val));
                final_states.push((path.clone(), state.vars.clone()));
            }
            ForgeTerminator::Unreachable => {
                ub_paths = ast::Bool::or(&[&ub_paths, &path]);
            }
        }
    }

    SymExecResult {
        returns,
        final_states,
        ub_paths,
        unwind_paths,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use z3::ast::{Ast, BV};
    use z3::{Context, SatResult, Solver};

    use super::{SymValue, sym_exec, symbolic_value, values_equal};
    use crate::ir::inst::place;
    use crate::ir::inst::{
        BlockId, ForgeBinOp, ForgeBlock, ForgeBody, ForgeCastKind, ForgeOperand, ForgeRvalue,
        ForgeStmt, ForgeTerminator, ForgeUnOp, VarId,
    };
    use crate::ir::ty::{ForgeTy, IntKind, IntWidth};

    #[test]
    fn signed_division_and_remainder_overflow_at_every_width() {
        for width in [
            IntWidth::I8,
            IntWidth::I16,
            IntWidth::I32,
            IntWidth::I64,
            IntWidth::I128,
        ] {
            let ty = ForgeTy::Int(width, IntKind::Signed);
            let min = super::signed_min(&ty);
            let neg_one = super::bv_all_ones(width.bits());
            for op in [ForgeBinOp::Div, ForgeBinOp::Rem] {
                let overflow = super::eval_binop(op, &ty, &min, &neg_one);
                assert_eq!(overflow.ub.simplify().as_bool(), Some(true));
                let safe =
                    super::eval_binop(op, &ty, &min.bvadd(BV::from_u64(1, width.bits())), &neg_one);
                assert_eq!(safe.ub.simplify().as_bool(), Some(false));
            }
        }
    }

    #[test]
    fn transmute_uses_memory_offsets_in_both_endiannesses() {
        let scalar = |bits| super::ForgeLayout {
            bits,
            fields: vec![],
        };
        let layout = super::ForgeLayout {
            bits: 32,
            fields: vec![(16, scalar(8)), (0, scalar(16)), (24, scalar(8))],
        };
        let ty = ForgeTy::Aggregate(vec![
            ForgeTy::Int(IntWidth::I8, IntKind::Unsigned),
            ForgeTy::Int(IntWidth::I16, IntKind::Unsigned),
            ForgeTy::Int(IntWidth::I8, IntKind::Unsigned),
        ]);
        let value = SymValue::Aggregate(vec![
            SymValue::Scalar(BV::from_u64(1, 8)),
            SymValue::Scalar(BV::from_u64(0x0203, 16)),
            SymValue::Scalar(BV::from_u64(4, 8)),
        ]);
        for (little, expected) in [(true, 0x04010203), (false, 0x02030104)] {
            let packed = super::pack_value_bits(value.clone(), &ty, &layout, little);
            assert_eq!(packed.simplify().as_u64(), Some(expected));
            let unpacked = super::unpack_value_bits(&packed, &ty, &layout, little);
            assert_eq!(
                values_equal(&value, &unpacked).simplify().as_bool(),
                Some(true)
            );
        }
    }

    fn single_block_body(ret_ty: ForgeTy, param_ty: ForgeTy, rvalue: ForgeRvalue) -> ForgeBody {
        ForgeBody {
            locals: vec![(VarId(0), ret_ty), (VarId(1), param_ty)],
            param_count: 1,
            blocks: vec![ForgeBlock {
                id: BlockId(0),
                stmts: vec![ForgeStmt::Assign(place(0), rvalue)],
                terminator: ForgeTerminator::Return,
            }],
        }
    }

    #[test]
    fn sym_exec_uses_logical_shift_for_unsigned_values() {
        let ty = ForgeTy::Int(IntWidth::I8, IntKind::Unsigned);
        let body = single_block_body(
            ty.clone(),
            ty.clone(),
            ForgeRvalue::BinaryOp {
                op: ForgeBinOp::Shr,
                lhs: ForgeOperand::Place(place(1)),
                rhs: ForgeOperand::Const(crate::ir::inst::ForgeConst::Int(1, ty.clone())),
                ty: ty.clone(),
            },
        );

        Context::thread_local();
        let param = symbolic_value(&ty, "x");
        let result = sym_exec(&body, &HashMap::from([(VarId(1), param.clone())]));
        let (path, ret) = &result.returns[0];
        let SymValue::Scalar(param) = param else {
            panic!("expected scalar param")
        };
        let SymValue::Scalar(ret) = ret else {
            panic!("expected scalar return")
        };

        let solver = Solver::new();
        solver.assert(path);
        solver.assert(param.eq(BV::from_u64(0x80, 8)));
        solver.assert(ret.eq(BV::from_u64(0x40, 8)));
        assert_eq!(solver.check(), SatResult::Sat);

        solver.reset();
        solver.assert(path);
        solver.assert(param.eq(BV::from_u64(0x80, 8)));
        solver.assert(ret.eq(BV::from_u64(0xC0, 8)));
        assert_eq!(solver.check(), SatResult::Unsat);
    }

    #[test]
    fn oversized_shift_does_not_report_ub() {
        let ty = ForgeTy::Int(IntWidth::I8, IntKind::Unsigned);
        let body = single_block_body(
            ty.clone(),
            ty.clone(),
            ForgeRvalue::BinaryOp {
                op: ForgeBinOp::Shl,
                lhs: ForgeOperand::Const(crate::ir::inst::ForgeConst::Int(1, ty.clone())),
                rhs: ForgeOperand::Place(place(1)),
                ty: ty.clone(),
            },
        );

        Context::thread_local();
        let param = symbolic_value(&ty, "x");
        let result = sym_exec(&body, &HashMap::from([(VarId(1), param.clone())]));
        let SymValue::Scalar(param) = param else {
            panic!("expected scalar param")
        };

        let solver = Solver::new();
        solver.assert(&result.ub_paths);
        solver.assert(param.eq(BV::from_u64(8, 8)));
        assert_eq!(solver.check(), SatResult::Unsat);
    }

    #[test]
    fn shift_amount_uses_machine_mask() {
        let lhs_ty = ForgeTy::Int(IntWidth::I16, IntKind::Signed);
        let rhs_ty = ForgeTy::Int(IntWidth::I128, IntKind::Signed);
        let body = ForgeBody {
            locals: vec![(VarId(0), lhs_ty.clone()), (VarId(1), rhs_ty.clone())],
            param_count: 1,
            blocks: vec![ForgeBlock {
                id: BlockId(0),
                stmts: vec![ForgeStmt::Assign(
                    place(0),
                    ForgeRvalue::BinaryOp {
                        op: ForgeBinOp::Shl,
                        lhs: ForgeOperand::Const(crate::ir::inst::ForgeConst::Int(
                            18679,
                            lhs_ty.clone(),
                        )),
                        rhs: ForgeOperand::Place(place(1)),
                        ty: lhs_ty.clone(),
                    },
                )],
                terminator: ForgeTerminator::Return,
            }],
        };

        Context::thread_local();
        let param = symbolic_value(&rhs_ty, "x");
        let result = sym_exec(&body, &HashMap::from([(VarId(1), param.clone())]));
        let (path, ret) = &result.returns[0];
        let SymValue::Scalar(param) = param else {
            panic!("expected scalar param")
        };
        let SymValue::Scalar(ret) = ret else {
            panic!("expected scalar return")
        };

        let solver = Solver::new();
        solver.assert(path);
        solver
            .assert(param.eq(BV::from_str(128, "68144735775423223540102237529186137220").unwrap()));
        solver.assert(ret.eq(BV::from_u64((-28816i16) as u16 as u64, 16)));
        assert_eq!(solver.check(), SatResult::Sat);
    }

    #[test]
    fn switch_int_handles_wide_targets() {
        let ty = ForgeTy::Int(IntWidth::I128, IntKind::Signed);
        let body = ForgeBody {
            locals: vec![(VarId(0), ty.clone()), (VarId(1), ty.clone())],
            param_count: 1,
            blocks: vec![
                ForgeBlock {
                    id: BlockId(0),
                    stmts: vec![],
                    terminator: ForgeTerminator::SwitchInt {
                        discr: ForgeOperand::Place(place(1)),
                        targets: vec![(271956399003478202655339028361449617458u128, BlockId(1))],
                        otherwise: BlockId(2),
                    },
                },
                ForgeBlock {
                    id: BlockId(1),
                    stmts: vec![ForgeStmt::Assign(
                        place(0),
                        ForgeRvalue::Use(ForgeOperand::Const(crate::ir::inst::ForgeConst::Int(
                            1,
                            ty.clone(),
                        ))),
                    )],
                    terminator: ForgeTerminator::Return,
                },
                ForgeBlock {
                    id: BlockId(2),
                    stmts: vec![ForgeStmt::Assign(
                        place(0),
                        ForgeRvalue::Use(ForgeOperand::Const(crate::ir::inst::ForgeConst::Int(
                            0,
                            ty.clone(),
                        ))),
                    )],
                    terminator: ForgeTerminator::Return,
                },
            ],
        };

        Context::thread_local();
        let param = symbolic_value(&ty, "x");
        let result = sym_exec(&body, &HashMap::from([(VarId(1), param.clone())]));
        let (path, ret) = &result.returns[0];
        let SymValue::Scalar(param) = param else {
            panic!("expected scalar param")
        };
        let SymValue::Scalar(ret) = ret else {
            panic!("expected scalar return")
        };

        let solver = Solver::new();
        solver.assert(path);
        solver.assert(
            param.eq(BV::from_str(128, "271956399003478202655339028361449617458").unwrap()),
        );
        solver.assert(ret.eq(BV::from_u64(1, 128)));
        assert_eq!(solver.check(), SatResult::Sat);
    }

    #[test]
    fn sym_exec_zero_extends_unsigned_int_casts() {
        let src_ty = ForgeTy::Int(IntWidth::I8, IntKind::Unsigned);
        let dst_ty = ForgeTy::Int(IntWidth::I16, IntKind::Signed);
        let body = single_block_body(
            dst_ty.clone(),
            src_ty.clone(),
            ForgeRvalue::Cast {
                kind: ForgeCastKind::IntToInt,
                operand: ForgeOperand::Place(place(1)),
                src_ty: src_ty.clone(),
                dst_ty: dst_ty.clone(),
            },
        );

        Context::thread_local();
        let param = symbolic_value(&src_ty, "x");
        let result = sym_exec(&body, &HashMap::from([(VarId(1), param.clone())]));
        let (path, ret) = &result.returns[0];
        let SymValue::Scalar(param) = param else {
            panic!("expected scalar param")
        };
        let SymValue::Scalar(ret) = ret else {
            panic!("expected scalar return")
        };

        let solver = Solver::new();
        solver.assert(path);
        solver.assert(param.eq(BV::from_u64(0xFF, 8)));
        solver.assert(ret.eq(BV::from_u64(0x00FF, 16)));
        assert_eq!(solver.check(), SatResult::Sat);

        solver.reset();
        solver.assert(path);
        solver.assert(param.eq(BV::from_u64(0xFF, 8)));
        solver.assert(ret.eq(BV::from_u64(0xFFFF, 16)));
        assert_eq!(solver.check(), SatResult::Unsat);
    }

    #[test]
    fn bool_not_flips_only_the_low_bit() {
        let bool_ty = ForgeTy::Int(IntWidth::I8, IntKind::Bool);
        let body = single_block_body(
            bool_ty.clone(),
            bool_ty.clone(),
            ForgeRvalue::UnaryOp {
                op: ForgeUnOp::Not,
                operand: ForgeOperand::Place(place(1)),
                ty: bool_ty.clone(),
            },
        );

        Context::thread_local();
        let param = symbolic_value(&bool_ty, "x");
        let result = sym_exec(&body, &HashMap::from([(VarId(1), param.clone())]));
        let (path, ret) = &result.returns[0];
        let SymValue::Scalar(param) = param else {
            panic!("expected scalar param")
        };
        let SymValue::Scalar(ret) = ret else {
            panic!("expected scalar return")
        };

        let solver = Solver::new();
        solver.assert(path);
        solver.assert(param.eq(BV::from_u64(1, 8)));
        solver.assert(ret.eq(BV::from_u64(0, 8)));
        assert_eq!(solver.check(), SatResult::Sat);

        solver.reset();
        solver.assert(path);
        solver.assert(param.eq(BV::from_u64(0, 8)));
        solver.assert(ret.eq(BV::from_u64(1, 8)));
        assert_eq!(solver.check(), SatResult::Sat);

        solver.reset();
        solver.assert(path);
        solver.assert(param.eq(BV::from_u64(1, 8)));
        solver.assert(ret.eq(BV::from_u64(0xFE, 8)));
        assert_eq!(solver.check(), SatResult::Unsat);
    }

    #[test]
    fn compares_aggregate_values_recursively() {
        Context::thread_local();
        let lhs = SymValue::Aggregate(vec![
            SymValue::Scalar(BV::from_u64(1, 8)),
            SymValue::Scalar(BV::from_u64(2, 8)),
        ]);
        let rhs = SymValue::Aggregate(vec![
            SymValue::Scalar(BV::from_u64(1, 8)),
            SymValue::Scalar(BV::from_u64(3, 8)),
        ]);

        let solver = Solver::new();
        solver.assert(values_equal(&lhs, &rhs));
        assert_eq!(solver.check(), SatResult::Unsat);
    }

    #[test]
    fn add_unchecked_reports_overflow_as_ub() {
        let ty = ForgeTy::Int(IntWidth::I8, IntKind::Signed);
        let body = single_block_body(
            ty.clone(),
            ty.clone(),
            ForgeRvalue::BinaryOp {
                op: ForgeBinOp::AddUnchecked,
                lhs: ForgeOperand::Place(place(1)),
                rhs: ForgeOperand::Const(crate::ir::inst::ForgeConst::Int(1, ty.clone())),
                ty: ty.clone(),
            },
        );

        Context::thread_local();
        let param = symbolic_value(&ty, "x");
        let result = sym_exec(&body, &HashMap::from([(VarId(1), param.clone())]));
        let SymValue::Scalar(param) = param else {
            panic!("expected scalar param")
        };

        let solver = Solver::new();
        solver.assert(&result.ub_paths);
        solver.assert(param.eq(BV::from_u64(0x7f, 8)));
        assert_eq!(solver.check(), SatResult::Sat);
    }

    #[test]
    fn add_with_overflow_returns_sum_and_flag() {
        let ret_ty = ForgeTy::Aggregate(vec![
            ForgeTy::Int(IntWidth::I8, IntKind::Unsigned),
            ForgeTy::Int(IntWidth::I8, IntKind::Unsigned),
        ]);
        let param_ty = ForgeTy::Int(IntWidth::I8, IntKind::Unsigned);
        let body = ForgeBody {
            locals: vec![
                (VarId(0), ret_ty.clone()),
                (VarId(1), param_ty.clone()),
                (VarId(2), param_ty.clone()),
            ],
            param_count: 2,
            blocks: vec![ForgeBlock {
                id: BlockId(0),
                stmts: vec![ForgeStmt::Assign(
                    place(0),
                    ForgeRvalue::BinaryOp {
                        op: ForgeBinOp::AddWithOverflow,
                        lhs: ForgeOperand::Place(place(1)),
                        rhs: ForgeOperand::Place(place(2)),
                        ty: param_ty.clone(),
                    },
                )],
                terminator: ForgeTerminator::Return,
            }],
        };

        Context::thread_local();
        let x = symbolic_value(&param_ty, "x");
        let y = symbolic_value(&param_ty, "y");
        let result = sym_exec(
            &body,
            &HashMap::from([(VarId(1), x.clone()), (VarId(2), y.clone())]),
        );
        let (path, ret) = &result.returns[0];
        let SymValue::Scalar(x) = x else {
            panic!("expected scalar x")
        };
        let SymValue::Scalar(y) = y else {
            panic!("expected scalar y")
        };
        let SymValue::Aggregate(fields) = ret else {
            panic!("expected aggregate return")
        };
        let SymValue::Scalar(sum) = &fields[0] else {
            panic!("expected scalar sum")
        };
        let SymValue::Scalar(of) = &fields[1] else {
            panic!("expected scalar overflow flag")
        };

        let solver = Solver::new();
        solver.assert(path);
        solver.assert(x.eq(BV::from_u64(255, 8)));
        solver.assert(y.eq(BV::from_u64(1, 8)));
        solver.assert(sum.eq(BV::from_u64(0, 8)));
        solver.assert(of.eq(BV::from_u64(1, 8)));
        assert_eq!(solver.check(), SatResult::Sat);
    }

    #[test]
    fn sub_unchecked_reports_overflow_as_ub() {
        let ty = ForgeTy::Int(IntWidth::I8, IntKind::Signed);
        let body = single_block_body(
            ty.clone(),
            ty.clone(),
            ForgeRvalue::BinaryOp {
                op: ForgeBinOp::SubUnchecked,
                lhs: ForgeOperand::Place(place(1)),
                rhs: ForgeOperand::Const(crate::ir::inst::ForgeConst::Int(1, ty.clone())),
                ty: ty.clone(),
            },
        );

        Context::thread_local();
        let param = symbolic_value(&ty, "x");
        let result = sym_exec(&body, &HashMap::from([(VarId(1), param.clone())]));
        let SymValue::Scalar(param) = param else {
            panic!("expected scalar param")
        };

        let solver = Solver::new();
        solver.assert(&result.ub_paths);
        solver.assert(param.eq(BV::from_u64(0x80, 8)));
        assert_eq!(solver.check(), SatResult::Sat);
    }

    #[test]
    fn mul_unchecked_reports_overflow_as_ub() {
        let ty = ForgeTy::Int(IntWidth::I8, IntKind::Unsigned);
        let body = single_block_body(
            ty.clone(),
            ty.clone(),
            ForgeRvalue::BinaryOp {
                op: ForgeBinOp::MulUnchecked,
                lhs: ForgeOperand::Place(place(1)),
                rhs: ForgeOperand::Const(crate::ir::inst::ForgeConst::Int(2, ty.clone())),
                ty: ty.clone(),
            },
        );

        Context::thread_local();
        let param = symbolic_value(&ty, "x");
        let result = sym_exec(&body, &HashMap::from([(VarId(1), param.clone())]));
        let SymValue::Scalar(param) = param else {
            panic!("expected scalar param")
        };

        let solver = Solver::new();
        solver.assert(&result.ub_paths);
        solver.assert(param.eq(BV::from_u64(200, 8)));
        assert_eq!(solver.check(), SatResult::Sat);
    }

    #[test]
    fn sub_with_overflow_returns_diff_and_flag() {
        let ret_ty = ForgeTy::Aggregate(vec![
            ForgeTy::Int(IntWidth::I8, IntKind::Signed),
            ForgeTy::Int(IntWidth::I8, IntKind::Unsigned),
        ]);
        let param_ty = ForgeTy::Int(IntWidth::I8, IntKind::Signed);
        let body = ForgeBody {
            locals: vec![
                (VarId(0), ret_ty.clone()),
                (VarId(1), param_ty.clone()),
                (VarId(2), param_ty.clone()),
            ],
            param_count: 2,
            blocks: vec![ForgeBlock {
                id: BlockId(0),
                stmts: vec![ForgeStmt::Assign(
                    place(0),
                    ForgeRvalue::BinaryOp {
                        op: ForgeBinOp::SubWithOverflow,
                        lhs: ForgeOperand::Place(place(1)),
                        rhs: ForgeOperand::Place(place(2)),
                        ty: param_ty.clone(),
                    },
                )],
                terminator: ForgeTerminator::Return,
            }],
        };

        Context::thread_local();
        let x = symbolic_value(&param_ty, "x");
        let y = symbolic_value(&param_ty, "y");
        let result = sym_exec(
            &body,
            &HashMap::from([(VarId(1), x.clone()), (VarId(2), y.clone())]),
        );
        let (path, ret) = &result.returns[0];
        let SymValue::Scalar(x) = x else {
            panic!("expected scalar x")
        };
        let SymValue::Scalar(y) = y else {
            panic!("expected scalar y")
        };
        let SymValue::Aggregate(fields) = ret else {
            panic!("expected aggregate return")
        };
        let SymValue::Scalar(diff) = &fields[0] else {
            panic!("expected scalar diff")
        };
        let SymValue::Scalar(of) = &fields[1] else {
            panic!("expected scalar overflow flag")
        };

        let solver = Solver::new();
        solver.assert(path);
        solver.assert(x.eq(BV::from_u64(0x80, 8)));
        solver.assert(y.eq(BV::from_u64(1, 8)));
        solver.assert(diff.eq(BV::from_u64(0x7f, 8)));
        solver.assert(of.eq(BV::from_u64(1, 8)));
        assert_eq!(solver.check(), SatResult::Sat);
    }
}
