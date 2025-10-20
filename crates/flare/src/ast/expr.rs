#[derive(Debug, Clone)]
pub enum Expr<'src> {
    Let {
        name: &'src str,
        value: Box<Expr<'src>>,
    },

    Call {
        func: &'src str,
        args: Vec<Expr<'src>>,
    },

    Binary {
        left: Box<Expr<'src>>,
        op: BinOp,
        right: Box<Expr<'src>>,
    },

    Ident(&'src str),
    Number(f64),
    Return(Box<Expr<'src>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}
