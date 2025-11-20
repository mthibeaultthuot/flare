use super::Stmt;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub struct Program<'src> {
    pub items: Vec<Stmt<'src>>,
    pub span: Range<usize>,
}
