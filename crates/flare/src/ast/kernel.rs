use super::{Expr, Type};
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct KernelDef<'src> {
    pub name: &'src str,
    pub params: Vec<Param<'src>>,
    pub return_type: Option<Type<'src>>,
    pub body: Vec<Expr<'src>>,
    pub span: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct Param<'src> {
    pub name: &'src str,
    pub ty: Type<'src>,
}
