#[derive(Debug, Clone)]
pub enum Type<'src> {
    Tensor {
        dtype: &'src str,
        shape: Vec<&'src str>,
    },
    Scalar(&'src str),
}
