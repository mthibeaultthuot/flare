use std::ops::Range;

#[derive(Debug, Clone)]
pub struct ScheduleBlock<'src> {
    pub tile: Option<(i32, i32, i32)>,
    pub vectorize: Option<i32>,
    pub unroll: Option<i32>,
    pub threads: Option<(i32, i32)>,
    pub memory: Vec<(&'src str, &'src str)>,
    pub span: Range<usize>,
}
