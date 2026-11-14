#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntKind {
    Bool,
    Signed,
    Unsigned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntWidth {
    I1,
    I8,
    I16,
    I32,
    I64,
    I128,
}

impl IntWidth {
    pub fn bits(self) -> u32 {
        match self {
            IntWidth::I1 => 1,
            IntWidth::I8 => 8,
            IntWidth::I16 => 16,
            IntWidth::I32 => 32,
            IntWidth::I64 => 64,
            IntWidth::I128 => 128,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ForgeTy {
    Int(IntWidth, IntKind),
    Aggregate(Vec<ForgeTy>),
    Ref(Box<ForgeTy>),
}

impl ForgeTy {
    pub fn bits(&self) -> u32 {
        match self {
            ForgeTy::Int(w, _) => w.bits(),
            ForgeTy::Aggregate(_) | ForgeTy::Ref(_) => {
                panic!("non-scalar types do not have a single bit-width")
            }
        }
    }

    pub fn total_bits(&self) -> u32 {
        match self {
            ForgeTy::Int(w, _) => w.bits(),
            ForgeTy::Aggregate(fields) => fields.iter().map(ForgeTy::total_bits).sum(),
            ForgeTy::Ref(_) => {
                panic!("reference types do not have a Forge bit-layout")
            }
        }
    }

    pub fn is_signed(&self) -> bool {
        matches!(self, ForgeTy::Int(_, IntKind::Signed))
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, ForgeTy::Int(IntWidth::I8, IntKind::Bool))
    }

    pub fn fields(&self) -> Option<&[ForgeTy]> {
        match self {
            ForgeTy::Aggregate(fields) => Some(fields),
            ForgeTy::Int(_, _) | ForgeTy::Ref(_) => None,
        }
    }
}

pub fn sign_extend(bits: u128, bit_width: u32) -> i128 {
    let shift = 128 - bit_width;
    ((bits as i128) << shift) >> shift
}

/// Rust memory layout for a value used by bit reinterpretation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForgeLayout {
    pub bits: u32,
    pub fields: Vec<(u32, ForgeLayout)>,
}
