use crate::hir::{Program, Stmt};
use crate::mir::error::LoweringError;

pub struct MIR<'a> {
    pub program: Program<'a>,
}

impl<'a> MIR<'a> {
    pub fn new(program: Program<'a>) -> Self {
        Self { program }
    }

    pub fn launch_lowering(&self) -> Result<(), LoweringError> {
        self.lower_program()?;
        Ok(())
    }

    pub fn lower_program(&self) -> Result<(), LoweringError> {
        self.program
            .items
            .iter()
            .try_for_each(|stmt| self.lower_stmt(stmt.to_owned()))?;
        Ok(())
    }

    pub fn lower_stmt(&self, stmt: Stmt<'a>) -> Result<(), LoweringError> {
        match stmt {
            Stmt::Kernel(kernel) => self.lower_kernel(kernel)?,
            _ => panic!(""),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use flare::Flare;
    //
    // use super::*;
    // #[test]
    // fn test_launch_lowering() {
    //     let source = r#"
    //         kernel simple() {
    //             let i = 1;
    //         }
    //     "#;
    //     let ast = Flare::compile_from_string(source).unwrap();
    //     let mir = MIR::new(ast);
    //     mir.launch_lowering().unwrap();
    // }
}
