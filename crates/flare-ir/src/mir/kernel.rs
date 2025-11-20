use crate::hir::KernelDef;

use crate::mir::{core::MIR, error::LoweringError};

impl<'a> MIR<'a> {
    pub fn lower_kernel(&self, kernel: KernelDef<'a>) -> Result<(), LoweringError> {
        println!("{:?}", kernel);
        Ok(())
    }
}
