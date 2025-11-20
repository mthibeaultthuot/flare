use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub struct FusionBlock<'src> {
    pub targets: Vec<&'src str>, 
    pub strategy: Option<FusionStrategy>,
    pub barriers: Vec<&'src str>,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FusionStrategy {
    Elementwise,
    Inline,
    Auto,
}
