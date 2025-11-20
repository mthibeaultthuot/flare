use super::{Expr, Param, Type};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub struct KernelDef<'src> {
    pub name: &'src str,
    pub generic_params: Vec<&'src str>,
    pub params: Vec<Param<'src>>,
    pub return_type: Option<Type<'src>>,
    pub grid: Option<Vec<Expr<'src>>>,
    pub block: Option<Vec<Expr<'src>>>,
    pub shared_memory: Option<Vec<SharedMemoryDecl<'src>>>,
    pub compute: Option<Vec<Stmt<'src>>>,
    pub body: Vec<Stmt<'src>>,
    pub attributes: Vec<Attribute<'src>>,
    pub span: Range<usize>,
}

use super::Stmt;

#[derive(Debug, Clone, PartialEq)]
pub struct SharedMemoryDecl<'src> {
    pub name: &'src str,
    pub shape: Vec<Expr<'src>>,
    pub ty: Option<Type<'src>>,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute<'src> {
    pub name: &'src str,
    pub args: Vec<AttributeArg<'src>>,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeArg<'src> {
    Ident(&'src str),
    IntLiteral(i64),
    StringLiteral(String),
}
