use super::core::Parser;
use crate::lexer::token::TokenKind;
use crate::FlareError;
use flare_ir::hir::*;

impl<'src> Parser<'src> {
    pub(crate) fn parse_kernel(&mut self) -> Result<KernelDef<'src>, FlareError> {
        let start = self.expect(TokenKind::Kernel)?.span.start;
        let name_token = self.expect(TokenKind::Identifier(String::new()))?;
        let name_token_span = name_token.span.clone();
        let name = self.get_string_from_span(&name_token_span);

        let mut generic_params = Vec::new();
        if self.match_token(&TokenKind::Less) {
            loop {
                let generic_token = self.expect(TokenKind::Identifier(String::new()))?;
                let generic_span = generic_token.span.clone();
                generic_params.push(self.get_string_from_span(&generic_span));

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::Greater)?;
        }

        self.expect(TokenKind::LeftParen)?;
        let mut params = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                let param_start = self.peek().map(|t| t.span.start).unwrap_or(0);
                let param_name_token = self.expect(TokenKind::Identifier(String::new()))?;
                let param_name_token_span = param_name_token.span.clone();
                let param_name = self.get_string_from_span(&param_name_token_span);
                self.expect(TokenKind::Colon)?;
                let param_type = self.parse_type()?;
                let param_span = self.span_from(param_start);

                params.push(Param {
                    name: param_name,
                    ty: param_type.clone(),
                    span: param_span,
                });
                println!("{:?}", param_name);
                println!("{:?}", self.tokens[self.current]);

                if !self.match_token(&TokenKind::Comma) {
                    println!("{:?}", param_type);
                    break;
                }
            }
        }

        self.expect(TokenKind::RightParen)?;

        let return_type = if self.match_token(&TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(TokenKind::LeftBrace)?;
        let mut grid = None;
        let mut block = None;
        let mut shared_memory = None;
        let mut compute = None;
        let mut body = Vec::new();

        while !self.check(&TokenKind::RightBrace) && self.peek().is_some() {
            if self.check(&TokenKind::Grid) {
                grid = Some(self.parse_grid_block()?);
            } else if self.check(&TokenKind::Block) {
                block = Some(self.parse_block_config()?);
            } else if self.check(&TokenKind::SharedMemory) {
                shared_memory = Some(self.parse_shared_memory_block()?);
            } else if self.check(&TokenKind::Compute) {
                compute = Some(self.parse_compute_block()?);
            } else {
                body.push(self.parse_statement()?);
            }
        }

        self.expect(TokenKind::RightBrace)?;

        let span = self.span_from(start);
        Ok(KernelDef {
            name,
            generic_params,
            params,
            return_type,
            grid,
            block,
            shared_memory,
            compute,
            body,
            attributes: Vec::new(),
            span,
        })
    }

    fn parse_grid_block(&mut self) -> Result<Vec<Expr<'src>>, FlareError> {
        self.expect(TokenKind::Grid)?;
        self.expect(TokenKind::Colon)?;
        self.expect(TokenKind::LeftBracket)?;

        let mut dimensions = Vec::new();
        if !self.check(&TokenKind::RightBracket) {
            loop {
                dimensions.push(self.parse_expression()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(TokenKind::RightBracket)?;
        Ok(dimensions)
    }

    fn parse_block_config(&mut self) -> Result<Vec<Expr<'src>>, FlareError> {
        self.expect(TokenKind::Block)?;
        self.expect(TokenKind::Colon)?;
        self.expect(TokenKind::LeftBracket)?;

        let mut dimensions = Vec::new();
        if !self.check(&TokenKind::RightBracket) {
            loop {
                dimensions.push(self.parse_expression()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(TokenKind::RightBracket)?;
        Ok(dimensions)
    }

    fn parse_shared_memory_block(&mut self) -> Result<Vec<SharedMemoryDecl<'src>>, FlareError> {
        self.expect(TokenKind::SharedMemory)?;
        self.expect(TokenKind::LeftBrace)?;

        let mut decls = Vec::new();

        while !self.check(&TokenKind::RightBrace) && self.peek().is_some() {
            let decl_start = self.peek().map(|t| t.span.start).unwrap_or(0);
            let name_token = self.expect(TokenKind::Identifier(String::new()))?;
            let name_span = name_token.span.clone();
            let name = self.get_string_from_span(&name_span);

            self.expect(TokenKind::Colon)?;
            self.expect(TokenKind::LeftBracket)?;

            let mut shape = Vec::new();
            if !self.check(&TokenKind::RightBracket) {
                loop {
                    shape.push(self.parse_expression()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
            }

            self.expect(TokenKind::RightBracket)?;

            let span = self.span_from(decl_start);
            decls.push(SharedMemoryDecl {
                name,
                ty: None,
                shape,
                span,
            });
        }

        self.expect(TokenKind::RightBrace)?;
        Ok(decls)
    }

    fn parse_compute_block(&mut self) -> Result<Vec<Stmt<'src>>, FlareError> {
        self.expect(TokenKind::Compute)?;
        self.expect(TokenKind::LeftBrace)?;

        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) && self.peek().is_some() {
            statements.push(self.parse_statement()?);
        }

        self.expect(TokenKind::RightBrace)?;
        Ok(statements)
    }

    pub(crate) fn parse_attribute(&mut self) -> Result<Attribute<'src>, FlareError> {
        let start = self.expect(TokenKind::At)?.span.start;
        let name_token = self.advance()?;
        let name_span = name_token.span.clone();
        let name = match &name_token.kind {
            TokenKind::Identifier(_) => self.get_string_from_span(&name_span),
            TokenKind::FusionPoint => "fusion_point",
            TokenKind::Fusable => "fusable",
            TokenKind::FusionTransform => "fusion_transform",
            TokenKind::FusedKernel => "fused_kernel",
            TokenKind::Optimize => "optimize",
            TokenKind::AutoTune => "auto_tune",
            TokenKind::ScheduleAnnotation => "schedule",
            TokenKind::MemoryAnnotation => "memory",
            TokenKind::DependsOn => "depends_on",
            TokenKind::Independent => "independent",
            TokenKind::PreferParallel => "prefer_parallel",
            TokenKind::MustWait => "must_wait",
            TokenKind::DynamicDispatch => "dynamic_dispatch",
            TokenKind::PipelineDepth => "pipeline_depth",
            TokenKind::P2PTransferAnnotation => "p2p_transfer",
            TokenKind::AllReduceAnnotation => "all_reduce",
            _ => {
                return Err(FlareError::UnexpectedToken(format!(
                    "expected attribute name, found {:?}",
                    name_token.kind
                )))
            }
        };

        let mut args = Vec::new();

        if self.match_token(&TokenKind::LeftParen) {
            if !self.check(&TokenKind::RightParen) {
                loop {
                    let arg_token = self.advance()?;
                    let arg_span = arg_token.span.clone();
                    let arg = match &arg_token.kind {
                        TokenKind::Identifier(_) => {
                            AttributeArg::Ident(self.get_string_from_span(&arg_span))
                        }
                        TokenKind::IntLiteral(n) => AttributeArg::IntLiteral(*n),
                        TokenKind::StringLiteral(s) => AttributeArg::StringLiteral(s.clone()),
                        _ => {
                            return Err(FlareError::UnexpectedToken(format!(
                                "expected attribute argument, found {:?}",
                                arg_token.kind
                            )))
                        }
                    };
                    args.push(arg);

                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RightParen)?;
        }

        let span = self.span_from(start);
        Ok(Attribute { name, args, span })
    }
}
