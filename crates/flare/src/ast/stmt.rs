use super::{Expr, FusionBlock, KernelDef, ScheduleBlock, Type};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt<'src> {
    Kernel(KernelDef<'src>),
    Fusion(FusionBlock<'src>),
    Schedule(ScheduleBlock<'src>),

    Function {
        name: &'src str,
        params: Vec<Param<'src>>,
        return_type: Option<Type<'src>>,
        body: Box<Expr<'src>>,
        span: Range<usize>,
    },

    Let {
        name: &'src str,
        ty: Option<Type<'src>>,
        value: Expr<'src>,
        span: Range<usize>,
    },
    Var {
        name: &'src str,
        ty: Option<Type<'src>>,
        value: Option<Expr<'src>>,
        span: Range<usize>,
    },
    Const {
        name: &'src str,
        ty: Option<Type<'src>>,
        value: Expr<'src>,
        span: Range<usize>,
    },

    If {
        condition: Expr<'src>,
        then_branch: Box<Stmt<'src>>,
        else_branch: Option<Box<Stmt<'src>>>,
        span: Range<usize>,
    },
    While {
        condition: Expr<'src>,
        body: Box<Stmt<'src>>,
        span: Range<usize>,
    },
    For {
        var: &'src str,
        iterator: Expr<'src>,
        body: Box<Stmt<'src>>,
        span: Range<usize>,
    },

    Return {
        value: Option<Expr<'src>>,
        span: Range<usize>,
    },

    Expr(Expr<'src>),

    Block {
        statements: Vec<Stmt<'src>>,
        span: Range<usize>,
    },

    SyncThreads {
        span: Range<usize>,
    },

    LoadShared {
        dest: &'src str,
        src: Expr<'src>,
        span: Range<usize>,
    },

    TypeDef {
        name: &'src str,
        ty: Type<'src>,
        span: Range<usize>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param<'src> {
    pub name: &'src str,
    pub ty: Type<'src>,
    pub span: Range<usize>,
}

impl<'src> Stmt<'src> {
    pub fn span(&self) -> Range<usize> {
        match self {
            Stmt::Kernel(k) => k.span.clone(),
            Stmt::Fusion(f) => f.span.clone(),
            Stmt::Schedule(s) => s.span.clone(),
            Stmt::Function { span, .. }
            | Stmt::Let { span, .. }
            | Stmt::Var { span, .. }
            | Stmt::Const { span, .. }
            | Stmt::If { span, .. }
            | Stmt::While { span, .. }
            | Stmt::For { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::Block { span, .. }
            | Stmt::SyncThreads { span, .. }
            | Stmt::LoadShared { span, .. }
            | Stmt::TypeDef { span, .. } => span.clone(),
            Stmt::Expr(e) => e.span(),
        }
    }
}
