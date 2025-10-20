use super::Stmt;

#[derive(Debug, Clone)]
pub struct Program<'src> {
    pub items: Vec<Stmt<'src>>,
}
