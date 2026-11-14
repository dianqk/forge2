pub mod report;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

use z3::ast::{self, BV};
use z3::{Context, Params, SatResult, Solver};

use crate::ir::inst::*;
use crate::ir::ty::{ForgeTy, sign_extend};
use crate::smt::encode::{
    SymExecResult, SymPlace, SymValue, merge_values, sym_exec, symbolic_value, values_equal,
};

pub enum VerifyResult {
    Verified,
    Counterexample(String),
    Error(String),
}

pub struct ParamInfo {
    pub var_id: VarId,
    pub name: String,
    pub display: ParamDisplay,
}

pub enum ParamDisplay {
    Raw,
    Char,
    Enum { variants: Vec<(u128, String)> },
}

const DEFAULT_Z3_TIMEOUT_MS: u32 = 10_000;
static Z3_TIMEOUT_MS: AtomicU32 = AtomicU32::new(DEFAULT_Z3_TIMEOUT_MS);

pub fn set_z3_timeout_ms(timeout_ms: u32) {
    Z3_TIMEOUT_MS.store(timeout_ms, Ordering::Relaxed);
}

pub fn z3_timeout_ms() -> u32 {
    Z3_TIMEOUT_MS.load(Ordering::Relaxed)
}

pub fn verify(src_body: &ForgeBody, tgt_body: &ForgeBody) -> VerifyResult {
    verify_with_param_info(src_body, tgt_body, &[])
}

pub fn verify_with_param_info(
    src_body: &ForgeBody,
    tgt_body: &ForgeBody,
    param_info: &[ParamInfo],
) -> VerifyResult {
    // Validate that src and tgt have the same signature.
    if src_body.param_count != tgt_body.param_count {
        return VerifyResult::Error("parameter count mismatch".into());
    }
    if src_body.ret_ty() != tgt_body.ret_ty() {
        return VerifyResult::Error("return type mismatch".into());
    }
    for (s, t) in src_body.params().iter().zip(tgt_body.params().iter()) {
        if s.1 != t.1 {
            return VerifyResult::Error(format!("param type mismatch: {:?} vs {:?}", s.1, t.1));
        }
    }

    // Initialize thread-local Z3 context.
    Context::thread_local();
    let solver = Solver::new();
    let mut params = Params::new();
    params.set_u32("timeout", z3_timeout_ms());
    solver.set_params(&params);

    // Create shared symbolic variables for parameters.
    let mut initial_vars: HashMap<VarId, SymValue> = HashMap::new();
    let mut ref_param_targets: HashMap<VarId, (VarId, ForgeTy)> = HashMap::new();
    let mut next_hidden_var = src_body
        .locals
        .iter()
        .chain(tgt_body.locals.iter())
        .map(|(var_id, _)| var_id.0)
        .max()
        .unwrap_or(0)
        + 1;
    for (var_id, ty) in src_body.params() {
        let name = format!("param_{}", var_id.0);
        match ty {
            ForgeTy::Ref(inner) => {
                let hidden_var = VarId(next_hidden_var);
                next_hidden_var += 1;
                let pointee = symbolic_value(inner, &format!("{name}_pointee"));
                initial_vars.insert(hidden_var, pointee);
                initial_vars.insert(*var_id, SymValue::Ref(SymPlace::root(hidden_var)));
                ref_param_targets.insert(*var_id, (hidden_var, (**inner).clone()));
            }
            _ => {
                let value = symbolic_value(ty, name.as_str());
                initial_vars.insert(*var_id, value);
            }
        }
    }

    // Symbolically execute both functions.
    let src_result = sym_exec(src_body, &initial_vars);
    let tgt_result = sym_exec(tgt_body, &initial_vars);

    // Check 1: Source reaches UB on all inputs.
    match check_source_always_ub(&solver, &src_result) {
        Ok(Some(msg)) => return VerifyResult::Counterexample(msg),
        Ok(None) => {}
        Err(msg) => return VerifyResult::Error(msg),
    }
    solver.reset();

    // Check 2: Target reaches UB where source doesn't.
    match check_target_ub(
        &solver,
        &src_result,
        &tgt_result,
        &initial_vars,
        src_body.params(),
        param_info,
        &ref_param_targets,
    ) {
        Ok(Some(msg)) => return VerifyResult::Counterexample(msg),
        Ok(None) => {}
        Err(msg) => return VerifyResult::Error(msg),
    }
    solver.reset();

    // Check 3: Unwind behavior matches on defined executions.
    match check_unwind_behavior(
        &solver,
        &src_result,
        &tgt_result,
        &initial_vars,
        src_body.params(),
        param_info,
        &ref_param_targets,
    ) {
        Ok(Some(msg)) => return VerifyResult::Counterexample(msg),
        Ok(None) => {}
        Err(msg) => return VerifyResult::Error(msg),
    }
    solver.reset();

    // Check 4: Return values match when both return.
    match check_return_values(
        &solver,
        &src_result,
        &tgt_result,
        &initial_vars,
        src_body,
        param_info,
        &ref_param_targets,
    ) {
        Ok(Some(msg)) => return VerifyResult::Counterexample(msg),
        Ok(None) => {}
        Err(msg) => return VerifyResult::Error(msg),
    }

    solver.reset();

    // Check 5: Reference parameter mutations match when both return.
    match check_param_mutations(
        &solver,
        &src_result,
        &tgt_result,
        &initial_vars,
        src_body.params(),
        param_info,
        &ref_param_targets,
    ) {
        Ok(Some(msg)) => return VerifyResult::Counterexample(msg),
        Ok(None) => {}
        Err(msg) => return VerifyResult::Error(msg),
    }

    VerifyResult::Verified
}

fn check_source_always_ub(solver: &Solver, src: &SymExecResult) -> Result<Option<String>, String> {
    solver.assert(src.ub_paths.not());

    match solver.check() {
        SatResult::Unsat => Ok(Some(String::from("Source always reaches unreachable.\n"))),
        SatResult::Sat => Ok(None),
        SatResult::Unknown => Err(unknown_solver_error(solver, "source-always-ub")),
    }
}

fn check_target_ub(
    solver: &Solver,
    src: &SymExecResult,
    tgt: &SymExecResult,
    initial_vars: &HashMap<VarId, SymValue>,
    param_order: &[(VarId, crate::ir::ty::ForgeTy)],
    param_info: &[ParamInfo],
    ref_param_targets: &HashMap<VarId, (VarId, ForgeTy)>,
) -> Result<Option<String>, String> {
    let constraint = ast::Bool::and(&[&tgt.ub_paths, &src.ub_paths.not()]);
    solver.assert(&constraint);

    match solver.check() {
        SatResult::Sat => {
            let model = solver.get_model().unwrap();
            let mut msg = String::from("Target reaches unreachable where source does not.\n");
            append_param_model(
                &mut msg,
                &model,
                initial_vars,
                param_order,
                param_info,
                ref_param_targets,
            );
            Ok(Some(msg))
        }
        SatResult::Unsat => Ok(None),
        SatResult::Unknown => Err(unknown_solver_error(solver, "target-ub")),
    }
}

fn check_return_values(
    solver: &Solver,
    src: &SymExecResult,
    tgt: &SymExecResult,
    initial_vars: &HashMap<VarId, SymValue>,
    source: &ForgeBody,
    param_info: &[ParamInfo],
    ref_param_targets: &HashMap<VarId, (VarId, ForgeTy)>,
) -> Result<Option<String>, String> {
    let ret_ty = source.ret_ty();
    let Some(src_ret) = build_return_expr(&src.returns) else {
        return Ok(None);
    };
    let Some(tgt_ret) = build_return_expr(&tgt.returns) else {
        return Ok(None);
    };
    let (src_ret_val, src_reachable) = src_ret;
    let (tgt_ret_val, tgt_reachable) = tgt_ret;

    let values_differ = values_equal(&src_ret_val, &tgt_ret_val).not();
    let constraint = ast::Bool::and(&[
        &src_reachable,
        &tgt_reachable,
        &src.ub_paths.not(),
        &values_differ,
    ]);
    solver.assert(&constraint);

    match solver.check() {
        SatResult::Sat => {
            let model = solver.get_model().unwrap();
            let mut msg = String::from("Return value mismatch:\n");
            append_param_model(
                &mut msg,
                &model,
                initial_vars,
                source.params(),
                param_info,
                ref_param_targets,
            );
            if let Some(src_val) = eval_model_value(&model, &src_ret_val) {
                msg.push_str(&format!(
                    "  src returns: {}\n",
                    format_value(None, &ret_ty, &src_val)
                ));
            }
            if let Some(tgt_val) = eval_model_value(&model, &tgt_ret_val) {
                msg.push_str(&format!(
                    "  tgt returns: {}\n",
                    format_value(None, &ret_ty, &tgt_val)
                ));
            }

            Ok(Some(msg))
        }
        SatResult::Unsat => Ok(None),
        SatResult::Unknown => Err(unknown_solver_error(solver, "return-values")),
    }
}

fn check_unwind_behavior(
    solver: &Solver,
    src: &SymExecResult,
    tgt: &SymExecResult,
    initial_vars: &HashMap<VarId, SymValue>,
    param_order: &[(VarId, crate::ir::ty::ForgeTy)],
    param_info: &[ParamInfo],
    ref_param_targets: &HashMap<VarId, (VarId, ForgeTy)>,
) -> Result<Option<String>, String> {
    let target_only_unwind = ast::Bool::and(&[
        &tgt.unwind_paths,
        &src.ub_paths.not(),
        &src.unwind_paths.not(),
    ]);
    solver.assert(&target_only_unwind);

    match solver.check() {
        SatResult::Sat => {
            let model = solver.get_model().unwrap();
            let mut msg = String::from("Target unwinds where source does not.\n");
            append_param_model(
                &mut msg,
                &model,
                initial_vars,
                param_order,
                param_info,
                ref_param_targets,
            );
            return Ok(Some(msg));
        }
        SatResult::Unsat => solver.reset(),
        SatResult::Unknown => {
            return Err(unknown_solver_error(solver, "target-only-unwind"));
        }
    }

    let source_only_unwind = ast::Bool::and(&[
        &src.unwind_paths,
        &src.ub_paths.not(),
        &tgt.unwind_paths.not(),
    ]);
    solver.assert(&source_only_unwind);

    match solver.check() {
        SatResult::Sat => {
            let model = solver.get_model().unwrap();
            let mut msg = String::from("Source unwinds where target does not.\n");
            append_param_model(
                &mut msg,
                &model,
                initial_vars,
                param_order,
                param_info,
                ref_param_targets,
            );
            Ok(Some(msg))
        }
        SatResult::Unsat => Ok(None),
        SatResult::Unknown => Err(unknown_solver_error(solver, "source-only-unwind")),
    }
}

fn check_param_mutations(
    solver: &Solver,
    src: &SymExecResult,
    tgt: &SymExecResult,
    initial_vars: &HashMap<VarId, SymValue>,
    param_order: &[(VarId, crate::ir::ty::ForgeTy)],
    param_info: &[ParamInfo],
    ref_param_targets: &HashMap<VarId, (VarId, ForgeTy)>,
) -> Result<Option<String>, String> {
    for (param_var, (hidden_var, pointee_ty)) in ref_param_targets {
        let Some((src_final, src_reachable)) = build_observed_expr(&src.final_states, *hidden_var)
        else {
            continue;
        };
        let Some((tgt_final, tgt_reachable)) = build_observed_expr(&tgt.final_states, *hidden_var)
        else {
            continue;
        };

        let differs = values_equal(&src_final, &tgt_final).not();
        let constraint = ast::Bool::and(&[
            &src_reachable,
            &tgt_reachable,
            &src.ub_paths.not(),
            &differs,
        ]);
        solver.assert(&constraint);

        match solver.check() {
            SatResult::Sat => {
                let model = solver.get_model().unwrap();
                let mut msg = String::from("Reference parameter mutation mismatch:\n");
                append_param_model(
                    &mut msg,
                    &model,
                    initial_vars,
                    param_order,
                    param_info,
                    ref_param_targets,
                );

                let param_name = param_info
                    .iter()
                    .find(|info| info.var_id == *param_var)
                    .map(|info| info.name.as_str())
                    .unwrap_or("param");

                if let Some(src_val) = eval_model_value(&model, &src_final) {
                    msg.push_str(&format!(
                        "  src writes through {param_name}: {}\n",
                        format_value(None, pointee_ty, &src_val)
                    ));
                }
                if let Some(tgt_val) = eval_model_value(&model, &tgt_final) {
                    msg.push_str(&format!(
                        "  tgt writes through {param_name}: {}\n",
                        format_value(None, pointee_ty, &tgt_val)
                    ));
                }
                return Ok(Some(msg));
            }
            SatResult::Unsat => solver.reset(),
            SatResult::Unknown => {
                return Err(unknown_solver_error(solver, "param-mutations"));
            }
        }
    }

    Ok(None)
}

fn unknown_solver_error(solver: &Solver, check_name: &str) -> String {
    let reason = solver
        .get_reason_unknown()
        .unwrap_or_else(|| "unknown reason".to_string());
    format!(
        "Z3 returned unknown during {check_name} check (timeout {} ms): {reason}",
        z3_timeout_ms()
    )
}

fn append_param_model(
    msg: &mut String,
    model: &z3::Model,
    initial_vars: &HashMap<VarId, SymValue>,
    param_order: &[(VarId, crate::ir::ty::ForgeTy)],
    param_info: &[ParamInfo],
    ref_param_targets: &HashMap<VarId, (VarId, ForgeTy)>,
) {
    let param_meta: HashMap<VarId, &ParamInfo> =
        param_info.iter().map(|info| (info.var_id, info)).collect();

    for (var_id, ty) in param_order {
        let info = param_meta.get(var_id).copied();
        let name = info.map(|info| info.name.as_str()).unwrap_or("param");

        if let Some((hidden_var, pointee_ty)) = ref_param_targets.get(var_id) {
            let Some(value) = initial_vars.get(hidden_var) else {
                continue;
            };
            if let Some(val) = eval_model_value(model, value) {
                let rendered = format_value(None, pointee_ty, &val);
                msg.push_str(&format!("  {name} = &mut {rendered}\n"));
            }
            continue;
        }

        let Some(value) = initial_vars.get(var_id) else {
            continue;
        };
        if let Some(val) = eval_model_value(model, value) {
            let rendered = format_value(info.map(|info| &info.display), ty, &val);
            msg.push_str(&format!("  {name} = {rendered}\n"));
        }
    }
}

fn build_return_expr(returns: &[(ast::Bool, SymValue)]) -> Option<(SymValue, ast::Bool)> {
    if returns.is_empty() {
        return None;
    }

    let mut result = zero_like(&returns[0].1);
    let mut any_reachable = ast::Bool::from_bool(false);

    for (path, val) in returns.iter().rev() {
        result = merge_values(path, val.clone(), result);
        any_reachable = ast::Bool::or(&[&any_reachable, path]);
    }

    Some((result, any_reachable))
}

fn build_observed_expr(
    states: &[(ast::Bool, HashMap<VarId, SymValue>)],
    var_id: VarId,
) -> Option<(SymValue, ast::Bool)> {
    if states.is_empty() {
        return None;
    }

    let mut result: Option<SymValue> = None;
    let mut any_reachable = ast::Bool::from_bool(false);

    for (path, vars) in states.iter().rev() {
        let value = vars.get(&var_id)?.clone();
        result = Some(match result {
            Some(current) => merge_values(path, value, current),
            None => value,
        });
        any_reachable = ast::Bool::or(&[&any_reachable, path]);
    }

    result.map(|value| (value, any_reachable))
}

fn format_value(display: Option<&ParamDisplay>, ty: &ForgeTy, value: &ConcreteValue) -> String {
    match display {
        Some(ParamDisplay::Char) => {
            let ConcreteValue::Scalar(scalar) = value else {
                return format_raw_value(ty, value);
            };
            if let Some(bits) = bv_to_u128(scalar)
                && let Some(ch) = char::from_u32(bits as u32)
            {
                return format!("{ch:?}");
            }
            format_raw_value(ty, value)
        }
        Some(ParamDisplay::Enum { variants }) => {
            let ConcreteValue::Scalar(scalar) = value else {
                return format_raw_value(ty, value);
            };
            if let Some(bits) = bv_to_u128(scalar)
                && let Some((_, name)) = variants.iter().find(|(discr, _)| *discr == bits)
            {
                return name.clone();
            }
            format_raw_value(ty, value)
        }
        _ => format_raw_value(ty, value),
    }
}

enum ConcreteValue {
    Scalar(BV),
    Aggregate(Vec<ConcreteValue>),
}

fn eval_model_value(model: &z3::Model, value: &SymValue) -> Option<ConcreteValue> {
    match value {
        SymValue::Scalar(bv) => model.eval(bv, true).map(ConcreteValue::Scalar),
        SymValue::Aggregate(fields) => fields
            .iter()
            .map(|field| eval_model_value(model, field))
            .collect::<Option<Vec<_>>>()
            .map(ConcreteValue::Aggregate),
        SymValue::Ref(_) | SymValue::UninitRef => None,
    }
}

fn zero_like(value: &SymValue) -> SymValue {
    match value {
        SymValue::Scalar(bv) => SymValue::Scalar(BV::from_i64(0, bv.get_size())),
        SymValue::Aggregate(fields) => SymValue::Aggregate(fields.iter().map(zero_like).collect()),
        SymValue::Ref(_) | SymValue::UninitRef => {
            unimplemented!("reference return values are not supported")
        }
    }
}

fn bv_to_u128(value: &BV) -> Option<u128> {
    let text = value.to_string();
    if let Some(hex) = text.strip_prefix("#x") {
        u128::from_str_radix(hex, 16).ok()
    } else if let Some(bin) = text.strip_prefix("#b") {
        u128::from_str_radix(bin, 2).ok()
    } else {
        text.parse::<u128>().ok()
    }
}

fn format_raw_value(ty: &ForgeTy, value: &ConcreteValue) -> String {
    match (ty, value) {
        (ForgeTy::Int(width, kind), ConcreteValue::Scalar(value)) => {
            let bits = match bv_to_u128(value) {
                Some(bits) => bits,
                None => return value.to_string(),
            };
            let bit_width = width.bits();
            match kind {
                crate::ir::ty::IntKind::Bool => match bits {
                    0 => String::from("false"),
                    1 => String::from("true"),
                    _ => bits.to_string(),
                },
                crate::ir::ty::IntKind::Unsigned => bits.to_string(),
                crate::ir::ty::IntKind::Signed => sign_extend(bits, bit_width).to_string(),
            }
        }
        (ForgeTy::Aggregate(fields), ConcreteValue::Aggregate(values)) => {
            let fields = fields
                .iter()
                .zip(values)
                .map(|(ty, value)| format_raw_value(ty, value))
                .collect::<Vec<_>>()
                .join(", ");
            format!("({fields})")
        }
        _ => String::from("<invalid>"),
    }
}

#[cfg(test)]
mod tests {
    use crate::ir::inst::place;
    use crate::ir::inst::{
        BlockId, ForgeBinOp, ForgeBlock, ForgeBody, ForgeConst, ForgeOperand, ForgeRvalue,
        ForgeStmt, ForgeTerminator, VarId,
    };
    use crate::ir::ty::{ForgeTy, IntKind, IntWidth};

    use super::{ConcreteValue, ParamDisplay, VerifyResult, bv_to_u128, format_value, verify};
    use z3::Context;
    use z3::ast::BV;

    #[test]
    fn verify_rejects_target_only_divide_by_zero() {
        let ty = ForgeTy::Int(IntWidth::I32, IntKind::Signed);

        let src = ForgeBody {
            locals: vec![(VarId(0), ty.clone())],
            param_count: 0,
            blocks: vec![ForgeBlock {
                id: BlockId(0),
                stmts: vec![ForgeStmt::Assign(
                    place(0),
                    ForgeRvalue::Use(ForgeOperand::Const(ForgeConst::Int(1, ty.clone()))),
                )],
                terminator: ForgeTerminator::Return,
            }],
        };

        let tgt = ForgeBody {
            locals: vec![(VarId(0), ty.clone())],
            param_count: 0,
            blocks: vec![ForgeBlock {
                id: BlockId(0),
                stmts: vec![ForgeStmt::Assign(
                    place(0),
                    ForgeRvalue::BinaryOp {
                        op: ForgeBinOp::Div,
                        lhs: ForgeOperand::Const(ForgeConst::Int(1, ty.clone())),
                        rhs: ForgeOperand::Const(ForgeConst::Int(0, ty.clone())),
                        ty: ty.clone(),
                    },
                )],
                terminator: ForgeTerminator::Return,
            }],
        };

        match verify(&src, &tgt) {
            VerifyResult::Counterexample(msg) => {
                assert!(msg.contains("Target reaches unreachable"));
            }
            VerifyResult::Verified => panic!("expected divide-by-zero UB to be rejected"),
            VerifyResult::Error(msg) => panic!("unexpected verifier error: {msg}"),
        }
    }

    #[test]
    fn formats_enum_param_values_from_discriminants() {
        Context::thread_local();
        let value = BV::from_u64(0xff, 8);
        let display = ParamDisplay::Enum {
            variants: vec![(0xff, "Foo::A".into()), (2, "Foo::B".into())],
        };
        let ty = ForgeTy::Int(IntWidth::I8, IntKind::Signed);

        assert_eq!(bv_to_u128(&value), Some(0xff));
        assert_eq!(
            format_value(Some(&display), &ty, &ConcreteValue::Scalar(value)),
            "Foo::A"
        );
    }

    #[test]
    fn formats_signed_and_unsigned_integers_in_decimal() {
        Context::thread_local();
        let signed_ty = ForgeTy::Int(IntWidth::I16, IntKind::Signed);
        let unsigned_ty = ForgeTy::Int(IntWidth::I16, IntKind::Unsigned);

        assert_eq!(
            format_value(
                None,
                &signed_ty,
                &ConcreteValue::Scalar(BV::from_u64(0xffff, 16))
            ),
            "-1"
        );
        assert_eq!(
            format_value(
                None,
                &unsigned_ty,
                &ConcreteValue::Scalar(BV::from_u64(0xffff, 16))
            ),
            "65535"
        );
    }

    #[test]
    fn formats_char_param_values() {
        Context::thread_local();
        let ty = ForgeTy::Int(IntWidth::I32, IntKind::Unsigned);

        assert_eq!(
            format_value(
                Some(&ParamDisplay::Char),
                &ty,
                &ConcreteValue::Scalar(BV::from_u64('A' as u64, 32))
            ),
            "'A'"
        );
    }
}
