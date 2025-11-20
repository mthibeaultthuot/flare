use super::Type;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'src> {
    IntLiteral(i64, Range<usize>),
    FloatLiteral(f64, Range<usize>),
    StringLiteral(String, Range<usize>),
    BoolLiteral(bool, Range<usize>),

    Ident(&'src str, Range<usize>),

    Binary {
        left: Box<Expr<'src>>,
        op: BinOp,
        right: Box<Expr<'src>>,
        span: Range<usize>,
    },

    Unary {
        op: UnOp,
        expr: Box<Expr<'src>>,
        span: Range<usize>,
    },

    Call {
        func: Box<Expr<'src>>,
        args: Vec<Expr<'src>>,
        span: Range<usize>,
    },

    Member {
        object: Box<Expr<'src>>,
        field: &'src str,
        span: Range<usize>,
    },

    Index {
        object: Box<Expr<'src>>,
        indices: Vec<Expr<'src>>,
        span: Range<usize>,
    },

    Range {
        start: Option<Box<Expr<'src>>>,
        end: Option<Box<Expr<'src>>>,
        span: Range<usize>,
    },

    Array {
        elements: Vec<Expr<'src>>,
        span: Range<usize>,
    },

    TensorInit {
        dtype: Type<'src>,
        shape: Vec<Expr<'src>>,
        span: Range<usize>,
    },

    If {
        condition: Box<Expr<'src>>,
        then_branch: Box<Expr<'src>>,
        else_branch: Option<Box<Expr<'src>>>,
        span: Range<usize>,
    },

    Block {
        statements: Vec<Stmt<'src>>,
        span: Range<usize>,
    },

    Assign {
        target: Box<Expr<'src>>,
        value: Box<Expr<'src>>,
        span: Range<usize>,
    },

    CompoundAssign {
        target: Box<Expr<'src>>,
        op: BinOp,
        value: Box<Expr<'src>>,
        span: Range<usize>,
    },

    Cast {
        expr: Box<Expr<'src>>,
        target_type: Type<'src>,
        span: Range<usize>,
    },

    ThreadIdx {
        dim: Option<&'src str>,
        span: Range<usize>,
    },
    BlockIdx {
        dim: Option<&'src str>,
        span: Range<usize>,
    },
    BlockDim {
        dim: Option<&'src str>,
        span: Range<usize>,
    },
}

use super::Stmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

impl<'src> Expr<'src> {
    pub fn span(&self) -> Range<usize> {
        match self {
            Expr::IntLiteral(_, span)
            | Expr::FloatLiteral(_, span)
            | Expr::StringLiteral(_, span)
            | Expr::BoolLiteral(_, span)
            | Expr::Ident(_, span)
            | Expr::Binary { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Call { span, .. }
            | Expr::Member { span, .. }
            | Expr::Index { span, .. }
            | Expr::Range { span, .. }
            | Expr::Array { span, .. }
            | Expr::TensorInit { span, .. }
            | Expr::If { span, .. }
            | Expr::Block { span, .. }
            | Expr::Assign { span, .. }
            | Expr::CompoundAssign { span, .. }
            | Expr::Cast { span, .. }
            | Expr::ThreadIdx { span, .. }
            | Expr::BlockIdx { span, .. }
            | Expr::BlockDim { span, .. } => span.clone(),
        }
    }
}
