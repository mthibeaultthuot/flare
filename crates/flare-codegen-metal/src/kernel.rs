use crate::error::{CodegenError, Result};
use crate::stmt::StmtGenerator;
use crate::types::TypeConverter;
use flare_ir::hir::*;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct KernelConfig {
    pub default_threadgroup_size: (u32, u32, u32),

    pub max_threads_per_threadgroup: u32,

    pub emit_debug: bool,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            default_threadgroup_size: (256, 1, 1),
            max_threads_per_threadgroup: 1024,
            emit_debug: false,
        }
    }
}

pub struct KernelGenerator {
    config: KernelConfig,

    stmt_gen: StmtGenerator,
}

impl KernelGenerator {
    pub fn new() -> Self {
        Self {
            config: KernelConfig::default(),
            stmt_gen: StmtGenerator::new(),
        }
    }

    pub fn with_config(config: KernelConfig) -> Self {
        Self {
            config,
            stmt_gen: StmtGenerator::new(),
        }
    }

    pub fn generate(
        &mut self,
        kernel: &KernelDef,
        schedule: Option<&ScheduleBlock>,
    ) -> Result<String> {
        let mut output = String::new();

        self.validate_kernel(kernel)?;

        let signature = self.generate_signature(kernel)?;
        writeln!(&mut output, "{}", signature)?;
        writeln!(&mut output, "{{")?;

        if let Some(shared_mem) = &kernel.shared_memory {
            for decl in shared_mem {
                let shared_code = self.generate_shared_memory(decl)?;
                writeln!(&mut output, "    {}", shared_code)?;
            }
            if !shared_mem.is_empty() {
                writeln!(&mut output)?;
            }
        }

        self.stmt_gen.set_indent(1);

        if let Some(compute_stmts) = &kernel.compute {
            for stmt in compute_stmts {
                let stmt_code = self.stmt_gen.generate(stmt)?;
                output.push_str(&stmt_code);
            }
        }

        for stmt in &kernel.body {
            let stmt_code = self.stmt_gen.generate(stmt)?;
            output.push_str(&stmt_code);
        }

        writeln!(&mut output, "}}")?;

        if let Some(sched) = schedule {
            output = self.apply_scheduling_hints(output, sched)?;
        }

        Ok(output)
    }

    fn generate_signature(&self, kernel: &KernelDef) -> Result<String> {
        let mut output = String::new();

        write!(&mut output, "kernel void {}", kernel.name)?;

        if !kernel.generic_params.is_empty() {
            return Err(CodegenError::unsupported_feature(
                "generic kernel parameters",
                kernel.span.clone(),
                Some(
                    "Metal kernels cannot be generic; use template specialization on host"
                        .to_string(),
                ),
            ));
        }

        write!(&mut output, "(")?;

        let mut param_index = 0;
        let mut params_code = Vec::new();

        for param in &kernel.params {
            let param_str = self.generate_parameter(param, param_index)?;
            params_code.push(param_str);
            param_index += 1;
        }

        params_code.push(
            "uint3 thread_position_in_threadgroup [[thread_position_in_threadgroup]]".to_string(),
        );
        params_code.push(
            "uint3 threadgroup_position_in_grid [[threadgroup_position_in_grid]]".to_string(),
        );
        params_code.push("uint3 threads_per_threadgroup [[threads_per_threadgroup]]".to_string());

        write!(
            &mut output,
            "{}",
            params_code.join(",\n                       ")
        )?;
        write!(&mut output, ")")?;

        Ok(output)
    }

    fn generate_parameter(&self, param: &Param, buffer_index: usize) -> Result<String> {
        let param_type = TypeConverter::convert(&param.ty, param.span.clone())?;

        let address_space = if param_type.as_str().contains("*") {
            "device"
        } else {
            ""
        };

        if address_space.is_empty() {
            Ok(format!(
                "{} {} [[buffer({})]]",
                param_type.as_str(),
                param.name,
                buffer_index
            ))
        } else {
            let type_str = param_type.as_str();
            if type_str.ends_with('*') {
                let base = &type_str[..type_str.len() - 1];
                Ok(format!(
                    "{} {} [[buffer({})]]",
                    base,
                    format!("*{}", param.name),
                    buffer_index
                ))
            } else {
                Ok(format!(
                    "{} {} [[buffer({})]]",
                    param_type.as_str(),
                    param.name,
                    buffer_index
                ))
            }
        }
    }

    fn generate_shared_memory(&self, decl: &SharedMemoryDecl) -> Result<String> {
        let ty_str = match &decl.ty {
            Some(ty) => TypeConverter::convert(ty, decl.span.clone())?
                .as_str()
                .to_string(),
            None => {
                return Err(CodegenError::invalid_memory_config(
                    "shared memory requires explicit type in Metal",
                    decl.span.clone(),
                ));
            }
        };

        if decl.shape.is_empty() {
            return Err(CodegenError::invalid_memory_config(
                "shared memory requires explicit shape",
                decl.span.clone(),
            ));
        }

        let mut expr_gen = crate::expr::ExprGenerator::new();
        let mut size_exprs = Vec::new();
        for dim in &decl.shape {
            size_exprs.push(expr_gen.generate(dim)?);
        }

        let array_spec = if size_exprs.len() == 1 {
            format!("[{}]", size_exprs[0])
        } else {
            format!("[{}]", size_exprs.join(" * "))
        };

        Ok(format!(
            "threadgroup {} {}{}",
            ty_str, decl.name, array_spec
        ))
    }

    fn validate_kernel(&self, kernel: &KernelDef) -> Result<()> {
        if let Some(grid) = &kernel.grid {
            if grid.len() > 3 {
                return Err(CodegenError::invalid_kernel_config(
                    "metal supports maximum 3D grid dims",
                    kernel.span.clone(),
                ));
            }
        }

        if let Some(block) = &kernel.block {
            if block.len() > 3 {
                return Err(CodegenError::invalid_kernel_config(
                    "metal supports maximum 3D threadgroup dims",
                    kernel.span.clone(),
                ));
            }

            if block.len() == 3 {}
        }

        Ok(())
    }

    fn apply_scheduling_hints(&self, code: String, schedule: &ScheduleBlock) -> Result<String> {
        let mut hints = String::new();

        writeln!(&mut hints, "// scheduling:")?;
        for directive in &schedule.directives {
            match directive {
                ScheduleDirective::Tile { x, y, z } => {
                    writeln!(&mut hints, "// tiling: ({}, {:?}, {:?})", x, y, z)?;
                }
                ScheduleDirective::Vectorize(factor) => {
                    writeln!(&mut hints, "// - vectorization factor: {}", factor)?;
                }
                ScheduleDirective::Unroll(factor) => {
                    writeln!(&mut hints, "// - unroll factor: {}", factor)?;
                }
                ScheduleDirective::Threads { x, y } => {
                    writeln!(&mut hints, "// - thread config: ({}, {:?})", x, y)?;
                }
                ScheduleDirective::Memory { var, location } => {
                    writeln!(
                        &mut hints,
                        "// - memory placement for '{}': {:?}",
                        var, location
                    )?;
                }
                ScheduleDirective::Stream(name) => {
                    writeln!(&mut hints, "// stream: {}", name)?;
                }
                ScheduleDirective::Pipeline { depth } => {
                    writeln!(&mut hints, "/ pipeline depth: {:?}", depth)?;
                }
                ScheduleDirective::Parallel => {
                    writeln!(&mut hints, "// parallel execution enabled")?;
                }
            }
        }

        hints.push_str(&code);
        Ok(hints)
    }

    pub fn get_threadgroup_size(
        &self,
        kernel: &KernelDef,
        schedule: Option<&ScheduleBlock>,
    ) -> (u32, u32, u32) {
        if let Some(sched) = schedule {
            for directive in &sched.directives {
                if let ScheduleDirective::Threads { x, y } = directive {
                    let y_val = y.unwrap_or(1);
                    return (*x as u32, y_val as u32, 1);
                }
            }
        }

        if let Some(block) = &kernel.block {
            match block.len() {
                1 => (256, 1, 1),
                2 => (16, 16, 1),
                3 => (8, 8, 8),
                _ => self.config.default_threadgroup_size,
            }
        } else {
            self.config.default_threadgroup_size
        }
    }
}

impl Default for KernelGenerator {
    fn default() -> Self {
        Self::new()
    }
}
