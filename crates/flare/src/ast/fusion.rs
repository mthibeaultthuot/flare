use std::ops::Range;

#[derive(Debug, Clone)]
pub struct FusionBlock<'src> {
    pub elementwise: bool,
    pub barriers: Vec<&'src str>,
    pub span: Range<usize>,
}
