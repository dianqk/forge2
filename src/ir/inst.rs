use crate::ir::ty::{ForgeLayout, ForgeTy};

/// A local variable identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(pub u32);

/// A basic block identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForgePlace {
    pub base: VarId,
    pub projections: Vec<ForgeProjection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ForgeProjection {
    Field(u32),
    Index(VarId),
    Deref,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForgeConst {
    Int(i128, ForgeTy),
    Aggregate(Vec<ForgeConst>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForgeOperand {
    Place(ForgePlace),
    Const(ForgeConst),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForgeBinOp {
    Add,
    AddUnchecked,
    AddWithOverflow,
    Sub,
    SubUnchecked,
    SubWithOverflow,
    Mul,
    MulUnchecked,
    MulWithOverflow,
    Div,
    Rem,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForgeUnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForgeCastKind {
    IntToInt,
    Transmute {
        little_endian: bool,
        src_layout: ForgeLayout,
        dst_layout: ForgeLayout,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForgeRvalue {
    Use(ForgeOperand),
    Ref(ForgePlace),
    ByteSwap {
        operand: ForgeOperand,
        ty: ForgeTy,
    },
    BinaryOp {
        op: ForgeBinOp,
        lhs: ForgeOperand,
        rhs: ForgeOperand,
        ty: ForgeTy,
    },
    UnaryOp {
        op: ForgeUnOp,
        operand: ForgeOperand,
        ty: ForgeTy,
    },
    Cast {
        kind: ForgeCastKind,
        operand: ForgeOperand,
        src_ty: ForgeTy,
        dst_ty: ForgeTy,
    },
    Aggregate(Vec<ForgeOperand>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForgeAssume {
    Bool(ForgeOperand),
    OneOf {
        operand: ForgeOperand,
        allowed: Vec<ForgeConst>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForgeStmt {
    Assign(ForgePlace, ForgeRvalue),
    Assume(ForgeAssume),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForgeTerminator {
    Goto(BlockId),
    Assert {
        cond: ForgeOperand,
        expected: bool,
        target: BlockId,
    },
    SwitchInt {
        discr: ForgeOperand,
        targets: Vec<(u128, BlockId)>,
        otherwise: BlockId,
    },
    Return,
    Unreachable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForgeBlock {
    pub id: BlockId,
    pub stmts: Vec<ForgeStmt>,
    pub terminator: ForgeTerminator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForgeBody {
    /// _0 is the return place, params start at _1.
    pub locals: Vec<(VarId, ForgeTy)>,
    pub param_count: usize,
    pub blocks: Vec<ForgeBlock>,
}

#[cfg(test)]
pub fn place(var: u32) -> ForgePlace {
    ForgePlace {
        base: VarId(var),
        projections: vec![],
    }
}

impl ForgeBody {
    pub fn ret_ty(&self) -> ForgeTy {
        self.locals[0].1.clone()
    }

    pub fn params(&self) -> &[(VarId, ForgeTy)] {
        &self.locals[1..=self.param_count]
    }
}
