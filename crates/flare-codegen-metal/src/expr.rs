use crate::error::{CodegenError, Result};
use crate::types::TypeConverter;
use flare::ast::{BinOp, Expr, UnOp};

pub struct ExprGenerator {
    indent_level: usize,
}

impl ExprGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    pub fn with_indent(indent_level: usize) -> Self {
        Self { indent_level }
    }

    pub fn generate(&mut self, expr: &Expr) -> Result<String> {
        match expr {
            Expr::IntLiteral(val, _) => Ok(val.to_string()),

            Expr::FloatLiteral(val, _) => {
                if val.fract() == 0.0 && !val.is_infinite() && !val.is_nan() {
                    Ok(format!("{}.0f", val))
                } else {
                    Ok(format!("{}f", val))
                }
            }

            Expr::StringLiteral(_val, span) => Err(CodegenError::unsupported_feature(
                "string literals",
                span.clone(),
                Some("metal compute shaders do not support string".to_string()),
            )),

            Expr::BoolLiteral(val, _) => Ok(val.to_string()),

            Expr::Ident(name, _) => Ok((*name).to_string()),

            Expr::Binary {
                left,
                op,
                right,
                span,
            } => self.generate_binary(left, *op, right, span.clone()),

            Expr::Unary { op, expr, span } => self.generate_unary(*op, expr, span.clone()),

            Expr::Call { func, args, span } => self.generate_call(func, args, span.clone()),

            Expr::Member {
                object,
                field,
                span,
            } => self.generate_member(object, field, span.clone()),

            Expr::Index {
                object,
                indices,
                span,
            } => self.generate_index(object, indices, span.clone()),

            Expr::Range {
                start: _,
                end: _,
                span,
            } => Err(CodegenError::unsupported_feature(
                "range expressions",
                span.clone(),
                Some("metal does not support range syntax".to_string()),
            )),

            Expr::Array { elements, span } => self.generate_array(elements, span.clone()),

            Expr::TensorInit {
                dtype: _,
                shape: _,
                span,
            } => Err(CodegenError::unsupported_feature(
                "tensor initialization",
                span.clone(),
                Some("tensors should be allocated on host and passed as buffers".to_string()),
            )),

            Expr::If {
                condition,
                then_branch,
                else_branch,
                span,
            } => self.generate_if_expr(condition, then_branch, else_branch.as_ref(), span.clone()),

            Expr::Block {
                statements: _,
                span,
            } => Err(CodegenError::unsupported_feature(
                "block expressions",
                span.clone(),
                Some("use block statements instead".to_string()),
            )),

            Expr::Assign { target, value, .. } => {
                let target_code = self.generate(target)?;
                let value_code = self.generate(value)?;
                Ok(format!("{} = {}", target_code, value_code))
            }

            Expr::CompoundAssign {
                target, op, value, ..
            } => {
                let target_code = self.generate(target)?;
                let value_code = self.generate(value)?;
                let op_str = Self::binop_to_string(*op);
                Ok(format!("{} {}= {}", target_code, op_str, value_code))
            }

            Expr::Cast {
                expr,
                target_type,
                span,
            } => {
                let expr_code = self.generate(expr)?;
                let type_code = TypeConverter::convert(target_type, span.clone())?;
                Ok(format!("{}({})", type_code.as_str(), expr_code))
            }

            Expr::ThreadIdx { dim, span } => self.generate_thread_idx(dim, span.clone()),

            Expr::BlockIdx { dim, span } => self.generate_block_idx(dim, span.clone()),

            Expr::BlockDim { dim, span } => self.generate_block_dim(dim, span.clone()),
        }
    }

    fn generate_binary(
        &mut self,
        left: &Expr,
        op: BinOp,
        right: &Expr,
        _span: std::ops::Range<usize>,
    ) -> Result<String> {
        let left_code = self.generate(left)?;
        let right_code = self.generate(right)?;
        let op_str = Self::binop_to_string(op);

        Ok(format!("({} {} {})", left_code, op_str, right_code))
    }

    fn generate_unary(
        &mut self,
        op: UnOp,
        expr: &Expr,
        _span: std::ops::Range<usize>,
    ) -> Result<String> {
        let expr_code = self.generate(expr)?;
        let op_str = match op {
            UnOp::Neg => "-",
            UnOp::Not => "!",
        };

        Ok(format!("({}{})", op_str, expr_code))
    }

    fn generate_call(
        &mut self,
        func: &Expr,
        args: &[Expr],
        _span: std::ops::Range<usize>,
    ) -> Result<String> {
        let func_code = self.generate(func)?;

        let mut args_code = Vec::new();
        for arg in args {
            args_code.push(self.generate(arg)?);
        }

        Ok(format!("{}({})", func_code, args_code.join(", ")))
    }

    fn generate_member(
        &mut self,
        object: &Expr,
        field: &str,
        _span: std::ops::Range<usize>,
    ) -> Result<String> {
        let object_code = self.generate(object)?;
        Ok(format!("{}.{}", object_code, field))
    }

    fn generate_index(
        &mut self,
        object: &Expr,
        indices: &[Expr],
        _span: std::ops::Range<usize>,
    ) -> Result<String> {
        let object_code = self.generate(object)?;

        if indices.is_empty() {
            return Ok(object_code);
        }

        if indices.len() == 1 {
            let index_code = self.generate(&indices[0])?;
            Ok(format!("{}[{}]", object_code, index_code))
        } else {
            let mut index_codes = Vec::new();
            for index in indices {
                index_codes.push(self.generate(index)?);
            }

            Ok(format!("{}[{}]", object_code, index_codes.join("][")))
        }
    }

    fn generate_array(
        &mut self,
        elements: &[Expr],
        _span: std::ops::Range<usize>,
    ) -> Result<String> {
        let mut elem_codes = Vec::new();
        for elem in elements {
            elem_codes.push(self.generate(elem)?);
        }

        Ok(format!("{{ {} }}", elem_codes.join(", ")))
    }

    fn generate_if_expr(
        &mut self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Box<Expr>>,
        span: std::ops::Range<usize>,
    ) -> Result<String> {
        let cond_code = self.generate(condition)?;
        let then_code = self.generate(then_branch)?;

        match else_branch {
            Some(else_expr) => {
                let else_code = self.generate(else_expr)?;
                Ok(format!("({} ? {} : {})", cond_code, then_code, else_code))
            }
            None => Err(CodegenError::expression_error(
                "if expression requires else branch for ternary operator",
                span,
            )),
        }
    }

    fn generate_thread_idx(
        &mut self,
        dim: &Option<&str>,
        span: std::ops::Range<usize>,
    ) -> Result<String> {
        match dim {
            Some("x") | Some("0") => Ok("thread_position_in_threadgroup.x".to_string()),
            Some("y") | Some("1") => Ok("thread_position_in_threadgroup.y".to_string()),
            Some("z") | Some("2") => Ok("thread_position_in_threadgroup.z".to_string()),
            None => Ok("thread_position_in_threadgroup".to_string()),
            Some(other) => Err(CodegenError::expression_error(
                format!("invalid thread_idx dimension: {}", other),
                span,
            )),
        }
    }

    fn generate_block_idx(
        &mut self,
        dim: &Option<&str>,
        span: std::ops::Range<usize>,
    ) -> Result<String> {
        match dim {
            Some("x") | Some("0") => Ok("threadgroup_position_in_grid.x".to_string()),
            Some("y") | Some("1") => Ok("threadgroup_position_in_grid.y".to_string()),
            Some("z") | Some("2") => Ok("threadgroup_position_in_grid.z".to_string()),
            None => Ok("threadgroup_position_in_grid".to_string()),
            Some(other) => Err(CodegenError::expression_error(
                format!("invalid block_idx dimension: {}", other),
                span,
            )),
        }
    }

    fn generate_block_dim(
        &mut self,
        dim: &Option<&str>,
        span: std::ops::Range<usize>,
    ) -> Result<String> {
        match dim {
            Some("x") | Some("0") => Ok("threads_per_threadgroup.x".to_string()),
            Some("y") | Some("1") => Ok("threads_per_threadgroup.y".to_string()),
            Some("z") | Some("2") => Ok("threads_per_threadgroup.z".to_string()),
            None => Ok("threads_per_threadgroup".to_string()),
            Some(other) => Err(CodegenError::expression_error(
                format!("invalid block_dim dimension: {}", other),
                span,
            )),
        }
    }

    fn binop_to_string(op: BinOp) -> &'static str {
        match op {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Equal => "==",
            BinOp::NotEqual => "!=",
            BinOp::Less => "<",
            BinOp::Greater => ">",
            BinOp::LessEqual => "<=",
            BinOp::GreaterEqual => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        }
    }
}

impl Default for ExprGenerator {
    fn default() -> Self {
        Self::new()
    }
}
