use std::fmt::Write;

use crate::ir::inst::{
    BlockId, ForgeAssume, ForgeBinOp, ForgeBody, ForgeCastKind, ForgeConst, ForgeOperand,
    ForgePlace, ForgeProjection, ForgeRvalue, ForgeStmt, ForgeTerminator, ForgeUnOp, VarId,
};
use crate::ir::ty::{ForgeTy, IntKind};

pub fn format_pair(suffix: Option<&str>, src: &ForgeBody, tgt: &ForgeBody) -> String {
    let mut out = String::new();
    let src_name = function_name("src", suffix);
    let tgt_name = function_name("tgt", suffix);
    write_named_body(&mut out, &src_name, src);
    out.push('\n');
    write_named_body(&mut out, &tgt_name, tgt);
    out
}

pub fn format_pass(name: &str, before: &ForgeBody, after: &ForgeBody) -> String {
    let mut out = String::new();
    write_named_body(&mut out, &format!("{name}_before"), before);
    out.push('\n');
    write_named_body(&mut out, &format!("{name}_after"), after);
    out
}

fn function_name(prefix: &str, suffix: Option<&str>) -> String {
    match suffix {
        Some(suffix) => format!("{prefix}_{suffix}"),
        None => prefix.to_string(),
    }
}

fn write_named_body(out: &mut String, name: &str, body: &ForgeBody) {
    let _ = write!(out, "fn {name}(");
    for (index, (var, ty)) in body.params().iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        let _ = write!(out, "{}: {}", fmt_var(var), fmt_ty(ty));
    }
    let _ = writeln!(out, ") -> {} {{", fmt_ty(&body.ret_ty()));

    let _ = writeln!(out, "  locals:");
    for (var, ty) in &body.locals {
        let _ = writeln!(out, "    {}: {}", fmt_var(var), fmt_ty(ty));
    }

    let _ = writeln!(out, "  blocks:");
    for block in &body.blocks {
        let _ = writeln!(out, "    {}:", fmt_block(block.id));
        for stmt in &block.stmts {
            let _ = writeln!(out, "      {}", fmt_stmt(stmt));
        }
        let _ = writeln!(out, "      {}", fmt_terminator(&block.terminator));
    }

    out.push('}');
    out.push('\n');
}

fn fmt_var(var: &VarId) -> String {
    format!("_{}", var.0)
}

fn fmt_block(block: BlockId) -> String {
    format!("bb{}", block.0)
}

fn fmt_ty(ty: &ForgeTy) -> String {
    match ty {
        ForgeTy::Int(_, IntKind::Bool) => "bool".to_string(),
        ForgeTy::Int(width, IntKind::Signed) => format!("i{}", width.bits()),
        ForgeTy::Int(width, IntKind::Unsigned) => format!("u{}", width.bits()),
        ForgeTy::Aggregate(fields) => {
            let fields = fields.iter().map(fmt_ty).collect::<Vec<_>>().join(", ");
            format!("({fields})")
        }
        ForgeTy::Ref(inner) => format!("&mut {}", fmt_ty(inner)),
    }
}

fn fmt_stmt(stmt: &ForgeStmt) -> String {
    match stmt {
        ForgeStmt::Assign(place, rvalue) => {
            format!("{} = {};", fmt_place(place), fmt_rvalue(rvalue))
        }
        ForgeStmt::Assume(assume) => format!("assume {};", fmt_assume(assume)),
    }
}

fn fmt_assume(assume: &ForgeAssume) -> String {
    match assume {
        ForgeAssume::Bool(operand) => fmt_operand(operand),
        ForgeAssume::OneOf { operand, allowed } => {
            let allowed = allowed.iter().map(fmt_const).collect::<Vec<_>>().join(", ");
            format!("{} in [{}]", fmt_operand(operand), allowed)
        }
    }
}

fn fmt_rvalue(rvalue: &ForgeRvalue) -> String {
    match rvalue {
        ForgeRvalue::Use(operand) => fmt_operand(operand),
        ForgeRvalue::Ref(place) => format!("&mut {}", fmt_place(place)),
        ForgeRvalue::ByteSwap { operand, ty } => {
            format!("bswap {} : {}", fmt_operand(operand), fmt_ty(ty))
        }
        ForgeRvalue::BinaryOp { op, lhs, rhs, ty } => {
            format!(
                "{} {} {} : {}",
                fmt_operand(lhs),
                fmt_binop(*op),
                fmt_operand(rhs),
                fmt_ty(ty)
            )
        }
        ForgeRvalue::UnaryOp { op, operand, .. } => {
            format!("{}{}", fmt_unop(*op), fmt_operand(operand))
        }
        ForgeRvalue::Cast {
            kind,
            operand,
            src_ty,
            dst_ty,
        } => format!(
            "{} {} : {} -> {}",
            fmt_cast_kind(kind),
            fmt_operand(operand),
            fmt_ty(src_ty),
            fmt_ty(dst_ty)
        ),
        ForgeRvalue::Aggregate(fields) => {
            let fields = fields
                .iter()
                .map(fmt_operand)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({fields})")
        }
    }
}

fn fmt_operand(operand: &ForgeOperand) -> String {
    match operand {
        ForgeOperand::Place(place) => fmt_place(place),
        ForgeOperand::Const(value) => fmt_const(value),
    }
}

fn fmt_place(place: &ForgePlace) -> String {
    let mut out = fmt_var(&place.base);
    for projection in &place.projections {
        match projection {
            ForgeProjection::Field(field) => {
                let _ = write!(out, ".{field}");
            }
            ForgeProjection::Index(var) => {
                let _ = write!(out, "[{}]", fmt_var(var));
            }
            ForgeProjection::Deref => out.insert(0, '*'),
        }
    }
    out
}

fn fmt_const(value: &ForgeConst) -> String {
    match value {
        ForgeConst::Int(value, ty) => format!("{value}:{}", fmt_ty(ty)),
        ForgeConst::Aggregate(fields) => {
            let fields = fields.iter().map(fmt_const).collect::<Vec<_>>().join(", ");
            format!("({fields})")
        }
    }
}

fn fmt_binop(op: ForgeBinOp) -> &'static str {
    match op {
        ForgeBinOp::Add => "+",
        ForgeBinOp::AddUnchecked => "+ unchecked",
        ForgeBinOp::AddWithOverflow => "+ overflow",
        ForgeBinOp::Sub => "-",
        ForgeBinOp::SubUnchecked => "- unchecked",
        ForgeBinOp::SubWithOverflow => "- overflow",
        ForgeBinOp::Mul => "*",
        ForgeBinOp::MulUnchecked => "* unchecked",
        ForgeBinOp::MulWithOverflow => "* overflow",
        ForgeBinOp::Div => "/",
        ForgeBinOp::Rem => "%",
        ForgeBinOp::BitAnd => "&",
        ForgeBinOp::BitOr => "|",
        ForgeBinOp::BitXor => "^",
        ForgeBinOp::Shl => "<<",
        ForgeBinOp::Shr => ">>",
        ForgeBinOp::Eq => "==",
        ForgeBinOp::Ne => "!=",
        ForgeBinOp::Lt => "<",
        ForgeBinOp::Le => "<=",
        ForgeBinOp::Gt => ">",
        ForgeBinOp::Ge => ">=",
    }
}

fn fmt_unop(op: ForgeUnOp) -> &'static str {
    match op {
        ForgeUnOp::Neg => "-",
        ForgeUnOp::Not => "!",
    }
}

fn fmt_cast_kind(kind: &ForgeCastKind) -> &'static str {
    match kind {
        ForgeCastKind::IntToInt => "cast",
        ForgeCastKind::Transmute { .. } => "transmute",
    }
}

fn fmt_terminator(terminator: &ForgeTerminator) -> String {
    match terminator {
        ForgeTerminator::Goto(target) => format!("goto {};", fmt_block(*target)),
        ForgeTerminator::Assert {
            cond,
            expected,
            target,
        } => {
            let expectation = if *expected { "" } else { "!" };
            format!(
                "assert {expectation}{} then {} else unwind;",
                fmt_operand(cond),
                fmt_block(*target)
            )
        }
        ForgeTerminator::SwitchInt {
            discr,
            targets,
            otherwise,
        } => {
            let targets = targets
                .iter()
                .map(|(value, block)| format!("{value} => {}", fmt_block(*block)))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "switchInt {} [{}] otherwise {};",
                fmt_operand(discr),
                targets,
                fmt_block(*otherwise)
            )
        }
        ForgeTerminator::Return => "return;".to_string(),
        ForgeTerminator::Unreachable => "unreachable;".to_string(),
    }
}
