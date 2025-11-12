use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum Type<'src> {
    Named(&'src str),

    I32,
    I64,
    U32,
    U64,
    F32,
    F64,
    Bool,

    Tensor {
        dtype: Box<Type<'src>>,
        shape: Vec<&'src str>,
    },
    Matrix {
        dtype: Box<Type<'src>>,
        rows: Option<&'src str>,
        cols: Option<&'src str>,
    },
    Vector {
        dtype: Box<Type<'src>>,
        len: Option<&'src str>,
    },

    Ptr(Box<Type<'src>>),

    Array {
        dtype: Box<Type<'src>>,
        size: Option<usize>,
    },
}

impl<'src> Type<'src> {
    pub fn span(&self) -> Range<usize> {
        0..0
    }
}
