use super::{FusionBlock, KernelDef, ScheduleBlock};

#[derive(Debug, Clone)]
pub enum Stmt<'src> {
    Kernel(KernelDef<'src>),
    Fusion(FusionBlock<'src>),
    Schedule(ScheduleBlock<'src>),
}
