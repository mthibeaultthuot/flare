use crate::error::{CodegenError, Result};
use crate::expr::ExprGenerator;
use crate::types::TypeConverter;
use flare_ir::hir::*;
use std::fmt::Write;

pub struct StmtGenerator {
    expr_gen: ExprGenerator,

    indent_level: usize,
}

impl StmtGenerator {
    pub fn new() -> Self {
        Self {
            expr_gen: ExprGenerator::new(),
            indent_level: 0,
        }
    }

    pub fn with_indent(indent_level: usize) -> Self {
        Self {
            expr_gen: ExprGenerator::with_indent(indent_level),
            indent_level,
        }
    }

    pub fn set_indent(&mut self, level: usize) {
        self.indent_level = level;
        self.expr_gen = ExprGenerator::with_indent(level);
    }

    pub fn indent(&mut self) {
        self.indent_level += 1;
        self.expr_gen = ExprGenerator::with_indent(self.indent_level);
    }

    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
            self.expr_gen = ExprGenerator::with_indent(self.indent_level);
        }
    }

    fn get_indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    pub fn generate(&mut self, stmt: &Stmt) -> Result<String> {
        match stmt {
            Stmt::Kernel(_) => Err(CodegenError::statement_error(
                "kernel statements should be handled by KernelGenerator",
                stmt.span(),
            )),

            Stmt::Fusion(_) | Stmt::Schedule(_) => Ok(String::new()),

            Stmt::Function {
                name,
                params,
                return_type,
                body,
                span,
            } => self.generate_function(name, params, return_type.as_ref(), body, span.clone()),

            Stmt::Let {
                name, ty, value, ..
            } => self.generate_let(name, ty.as_ref(), value),

            Stmt::Var {
                name, ty, value, ..
            } => self.generate_var(name, ty.as_ref(), value.as_ref()),

            Stmt::Const {
                name, ty, value, ..
            } => self.generate_const(name, ty.as_ref(), value),

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => self.generate_if(condition, then_branch, else_branch.as_ref()),

            Stmt::While {
                condition, body, ..
            } => self.generate_while(condition, body),

            Stmt::For {
                var,
                iterator,
                body,
                span,
            } => self.generate_for(var, iterator, body, span.clone()),

            Stmt::Return { value, .. } => self.generate_return(value.as_ref()),

            Stmt::Expr(expr) => {
                let expr_code = self.expr_gen.generate(expr)?;
                Ok(format!("{}{};\n", self.get_indent(), expr_code))
            }

            Stmt::Block { statements, .. } => self.generate_block(statements),

            Stmt::SyncThreads { .. } => Ok(format!(
                "{}threadgroup_barrier(mem_flags::mem_threadgroup);\n",
                self.get_indent()
            )),

            Stmt::LoadShared { dest, src, .. } => {
                let src_code = self.expr_gen.generate(src)?;
                Ok(format!("{}{} = {};\n", self.get_indent(), dest, src_code))
            }

            Stmt::TypeDef { .. } => Ok(String::new()),
        }
    }

    fn generate_function(
        &mut self,
        name: &str,
        params: &[Param],
        return_type: Option<&Type>,
        body: &Expr,
        span: std::ops::Range<usize>,
    ) -> Result<String> {
        let mut output = String::new();

        let ret_type = match return_type {
            Some(ty) => TypeConverter::convert(ty, span.clone())?
                .as_str()
                .to_string(),
            None => "void".to_string(),
        };

        let mut param_strs = Vec::new();
        for param in params {
            let param_type = TypeConverter::convert(&param.ty, param.span.clone())?;
            param_strs.push(format!("{} {}", param_type.as_str(), param.name));
        }

        write!(
            &mut output,
            "{}{} {}({})",
            self.get_indent(),
            ret_type,
            name,
            param_strs.join(", ")
        )?;

        let body_code = self.expr_gen.generate(body)?;
        writeln!(&mut output, " {{")?;
        self.indent();
        writeln!(&mut output, "{}return {};", self.get_indent(), body_code)?;
        self.dedent();
        writeln!(&mut output, "{}}}", self.get_indent())?;

        Ok(output)
    }

    fn generate_let(&mut self, name: &str, ty: Option<&Type>, value: &Expr) -> Result<String> {
        let value_code = self.expr_gen.generate(value)?;

        match ty {
            Some(t) => {
                let type_code = TypeConverter::convert(t, value.span())?;
                Ok(format!(
                    "{}const {} {} = {};\n",
                    self.get_indent(),
                    type_code.as_str(),
                    name,
                    value_code
                ))
            }
            None => Ok(format!(
                "{}const auto {} = {};\n",
                self.get_indent(),
                name,
                value_code
            )),
        }
    }

    fn generate_var(
        &mut self,
        name: &str,
        ty: Option<&Type>,
        value: Option<&Expr>,
    ) -> Result<String> {
        match (ty, value) {
            (Some(t), Some(v)) => {
                let type_code = TypeConverter::convert(t, v.span())?;
                let value_code = self.expr_gen.generate(v)?;
                Ok(format!(
                    "{}{} {} = {};\n",
                    self.get_indent(),
                    type_code.as_str(),
                    name,
                    value_code
                ))
            }
            (Some(t), None) => {
                let type_code = TypeConverter::convert(t, 0..0)?;
                Ok(format!(
                    "{}{} {};\n",
                    self.get_indent(),
                    type_code.as_str(),
                    name
                ))
            }
            (None, Some(v)) => {
                let value_code = self.expr_gen.generate(v)?;
                Ok(format!(
                    "{}auto {} = {};\n",
                    self.get_indent(),
                    name,
                    value_code
                ))
            }
            (None, None) => Err(CodegenError::statement_error(
                "var statement requires either type or value",
                0..0,
            )),
        }
    }

    fn generate_const(&mut self, name: &str, ty: Option<&Type>, value: &Expr) -> Result<String> {
        let value_code = self.expr_gen.generate(value)?;

        match ty {
            Some(t) => {
                let type_code = TypeConverter::convert(t, value.span())?;
                Ok(format!(
                    "{}constant {} {} = {};\n",
                    self.get_indent(),
                    type_code.as_str(),
                    name,
                    value_code
                ))
            }
            None => Ok(format!(
                "{}constant auto {} = {};\n",
                self.get_indent(),
                name,
                value_code
            )),
        }
    }

    fn generate_if(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Box<Stmt>>,
    ) -> Result<String> {
        let mut output = String::new();

        let cond_code = self.expr_gen.generate(condition)?;
        writeln!(&mut output, "{}if ({}) {{", self.get_indent(), cond_code)?;

        self.indent();
        let then_code = self.generate(then_branch)?;
        output.push_str(&then_code);
        self.dedent();

        if let Some(else_stmt) = else_branch {
            writeln!(&mut output, "{}}} else {{", self.get_indent())?;
            self.indent();
            let else_code = self.generate(else_stmt)?;
            output.push_str(&else_code);
            self.dedent();
        }

        writeln!(&mut output, "{}}}", self.get_indent())?;

        Ok(output)
    }

    fn generate_while(&mut self, condition: &Expr, body: &Stmt) -> Result<String> {
        let mut output = String::new();

        let cond_code = self.expr_gen.generate(condition)?;
        writeln!(&mut output, "{}while ({}) {{", self.get_indent(), cond_code)?;

        self.indent();
        let body_code = self.generate(body)?;
        output.push_str(&body_code);
        self.dedent();

        writeln!(&mut output, "{}}}", self.get_indent())?;

        Ok(output)
    }

    fn generate_for(
        &mut self,
        var: &str,
        iterator: &Expr,
        body: &Stmt,
        span: std::ops::Range<usize>,
    ) -> Result<String> {
        match iterator {
            Expr::Range { start, end, .. } => {
                let start_code = match start {
                    Some(s) => self.expr_gen.generate(s)?,
                    None => "0".to_string(),
                };

                let end_code = match end {
                    Some(e) => self.expr_gen.generate(e)?,
                    None => {
                        return Err(CodegenError::statement_error(
                            "for loop range requires end value",
                            span,
                        ));
                    }
                };

                let mut output = String::new();
                writeln!(
                    &mut output,
                    "{}for (int {} = {}; {} < {}; {}++) {{",
                    self.get_indent(),
                    var,
                    start_code,
                    var,
                    end_code,
                    var
                )?;

                self.indent();
                let body_code = self.generate(body)?;
                output.push_str(&body_code);
                self.dedent();

                writeln!(&mut output, "{}}}", self.get_indent())?;

                Ok(output)
            }
            _ => Err(CodegenError::statement_error(
                "for loop iterator must be a range expression in Metal codegen",
                span,
            )),
        }
    }

    fn generate_return(&mut self, value: Option<&Expr>) -> Result<String> {
        match value {
            Some(expr) => {
                let expr_code = self.expr_gen.generate(expr)?;
                Ok(format!("{}return {};\n", self.get_indent(), expr_code))
            }
            None => Ok(format!("{}return;\n", self.get_indent())),
        }
    }

    fn generate_block(&mut self, statements: &[Stmt]) -> Result<String> {
        let mut output = String::new();

        writeln!(&mut output, "{}{{", self.get_indent())?;
        self.indent();

        for stmt in statements {
            let stmt_code = self.generate(stmt)?;
            output.push_str(&stmt_code);
        }

        self.dedent();
        writeln!(&mut output, "{}}}", self.get_indent())?;

        Ok(output)
    }
}

impl Default for StmtGenerator {
    fn default() -> Self {
        Self::new()
    }
}
