use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduleBlock<'src> {
    pub target: Option<&'src str>, 
    pub directives: Vec<ScheduleDirective<'src>>,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScheduleDirective<'src> {
    Tile {
        x: i64,
        y: Option<i64>,
        z: Option<i64>,
    },
    Vectorize(i64),
    Unroll(i64),
    Threads {
        x: i64,
        y: Option<i64>,
    },
    Memory {
        var: &'src str,
        location: MemoryLocation<'src>,
    },
    Stream(&'src str),
    Pipeline {
        depth: Option<i64>,
    },
    Parallel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryLocation<'src> {
    Shared,
    Global,
    Local,
    Constant,
    Persistent,
    Temporary,
    Streaming,
    Named(&'src str),
}
