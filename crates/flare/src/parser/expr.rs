use super::core::Parser;
use crate::ast::*;
use crate::lexer::token::TokenKind;
use crate::FlareError;

impl<'src> Parser<'src> {
    pub(crate) fn parse_expression(&mut self) -> Result<Expr<'src>, FlareError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr<'src>, FlareError> {
        let expr = self.parse_logical_or()?;

        if let Some(token) = self.peek() {
            let start = expr.span().start;
            match &token.kind {
                TokenKind::Assign => {
                    self.advance()?;
                    let value = self.parse_assignment()?;
                    let span = self.span_from(start);
                    return Ok(Expr::Assign {
                        target: Box::new(expr),
                        value: Box::new(value),
                        span,
                    });
                }
                TokenKind::PlusAssign
                | TokenKind::MinusAssign
                | TokenKind::StarAssign
                | TokenKind::SlashAssign => {
                    let op = match &token.kind {
                        TokenKind::PlusAssign => BinOp::Add,
                        TokenKind::MinusAssign => BinOp::Sub,
                        TokenKind::StarAssign => BinOp::Mul,
                        TokenKind::SlashAssign => BinOp::Div,
                        _ => unreachable!(),
                    };
                    self.advance()?;
                    let value = self.parse_assignment()?;
                    let span = self.span_from(start);
                    return Ok(Expr::CompoundAssign {
                        target: Box::new(expr),
                        op,
                        value: Box::new(value),
                        span,
                    });
                }
                _ => {}
            }
        }

        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expr<'src>, FlareError> {
        let mut left = self.parse_logical_and()?;

        while self.match_token(&TokenKind::Or) {
            let start = left.span().start;
            let right = self.parse_logical_and()?;
            let span = self.span_from(start);
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expr<'src>, FlareError> {
        let mut left = self.parse_equality()?;

        while self.match_token(&TokenKind::And) {
            let start = left.span().start;
            let right = self.parse_equality()?;
            let span = self.span_from(start);
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr<'src>, FlareError> {
        let mut left = self.parse_comparison()?;

        while let Some(token) = self.peek() {
            let op = match &token.kind {
                TokenKind::Equal => BinOp::Equal,
                TokenKind::NotEqual => BinOp::NotEqual,
                _ => break,
            };
            self.advance()?;
            let start = left.span().start;
            let right = self.parse_comparison()?;
            let span = self.span_from(start);
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr<'src>, FlareError> {
        let mut left = self.parse_range()?;

        while let Some(token) = self.peek() {
            let op = match &token.kind {
                TokenKind::Less => BinOp::Less,
                TokenKind::Greater => BinOp::Greater,
                TokenKind::LessEqual => BinOp::LessEqual,
                TokenKind::GreaterEqual => BinOp::GreaterEqual,
                _ => break,
            };
            self.advance()?;
            let start = left.span().start;
            let right = self.parse_range()?;
            let span = self.span_from(start);
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_range(&mut self) -> Result<Expr<'src>, FlareError> {
        let start_expr = self.parse_term()?;

        if self.match_token(&TokenKind::DotDot) {
            let start = start_expr.span().start;
            let end = if self.check(&TokenKind::Semicolon)
                || self.check(&TokenKind::RightBracket)
                || self.check(&TokenKind::RightParen)
            {
                None
            } else {
                Some(Box::new(self.parse_term()?))
            };
            let span = self.span_from(start);
            return Ok(Expr::Range {
                start: Some(Box::new(start_expr)),
                end,
                span,
            });
        }

        Ok(start_expr)
    }

    fn parse_term(&mut self) -> Result<Expr<'src>, FlareError> {
        let mut left = self.parse_factor()?;

        while let Some(token) = self.peek() {
            let op = match &token.kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance()?;
            let start = left.span().start;
            let right = self.parse_factor()?;
            let span = self.span_from(start);
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr<'src>, FlareError> {
        let mut left = self.parse_unary()?;

        while let Some(token) = self.peek() {
            let op = match &token.kind {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance()?;
            let start = left.span().start;
            let right = self.parse_unary()?;
            let span = self.span_from(start);
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr<'src>, FlareError> {
        if let Some(token) = self.peek() {
            let start = token.span.start;
            match &token.kind {
                TokenKind::Minus => {
                    self.advance()?;
                    let expr = self.parse_unary()?;
                    let span = self.span_from(start);
                    return Ok(Expr::Unary {
                        op: UnOp::Neg,
                        expr: Box::new(expr),
                        span,
                    });
                }
                TokenKind::Not => {
                    self.advance()?;
                    let expr = self.parse_unary()?;
                    let span = self.span_from(start);
                    return Ok(Expr::Unary {
                        op: UnOp::Not,
                        expr: Box::new(expr),
                        span,
                    });
                }
                _ => {}
            }
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr<'src>, FlareError> {
        let mut expr = self.parse_primary()?;

        loop {
            if let Some(token) = self.peek() {
                let start = expr.span().start;
                match &token.kind {
                    TokenKind::LeftParen => {
                        self.advance()?;
                        let mut args = Vec::new();

                        if !self.check(&TokenKind::RightParen) {
                            loop {
                                args.push(self.parse_expression()?);
                                if !self.match_token(&TokenKind::Comma) {
                                    break;
                                }
                            }
                        }

                        self.expect(TokenKind::RightParen)?;
                        let span = self.span_from(start);
                        expr = Expr::Call {
                            func: Box::new(expr),
                            args,
                            span,
                        };
                    }
                    TokenKind::LeftBracket => {
                        self.advance()?;
                        let mut indices = Vec::new();

                        if !self.check(&TokenKind::RightBracket) {
                            loop {
                                indices.push(self.parse_expression()?);
                                if !self.match_token(&TokenKind::Comma) {
                                    break;
                                }
                            }
                        }

                        self.expect(TokenKind::RightBracket)?;
                        let span = self.span_from(start);
                        expr = Expr::Index {
                            object: Box::new(expr),
                            indices,
                            span,
                        };
                    }
                    TokenKind::Dot => {
                        self.advance()?;
                        let field_token = self.expect(TokenKind::Identifier(String::new()))?;
                        let field_span = field_token.span.clone();
                        let field = self.get_string_from_span(&field_span);
                        let span = self.span_from(start);
                        expr = Expr::Member {
                            object: Box::new(expr),
                            field,
                            span,
                        };
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr<'src>, FlareError> {
        let token = self.advance()?;
        let span = token.span.clone();

        match &token.kind {
            TokenKind::IntLiteral(n) => Ok(Expr::IntLiteral(*n, span)),
            TokenKind::FloatLiteral(f) => Ok(Expr::FloatLiteral(*f, span)),
            TokenKind::StringLiteral(s) => Ok(Expr::StringLiteral(s.clone(), span)),
            TokenKind::True => Ok(Expr::BoolLiteral(true, span)),
            TokenKind::False => Ok(Expr::BoolLiteral(false, span)),
            TokenKind::Identifier(_) => {
                let name = self.get_string_from_span(&span);
                Ok(Expr::Ident(name, span))
            }
            TokenKind::LeftParen => {
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RightParen)?;
                Ok(expr)
            }
            TokenKind::LeftBracket => {
                let start = span.start;
                let mut elements = Vec::new();

                if !self.check(&TokenKind::RightBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                }

                self.expect(TokenKind::RightBracket)?;
                let span = self.span_from(start);
                Ok(Expr::Array { elements, span })
            }
            TokenKind::If => {
                let start = span.start;
                let condition = Box::new(self.parse_expression()?);
                let then_branch = Box::new(self.parse_block_expr()?);
                let else_branch = if self.match_token(&TokenKind::Else) {
                    if self.check(&TokenKind::If) {
                        Some(Box::new(self.parse_expression()?))
                    } else {
                        Some(Box::new(self.parse_block_expr()?))
                    }
                } else {
                    None
                };
                let span = self.span_from(start);
                Ok(Expr::If {
                    condition,
                    then_branch,
                    else_branch,
                    span,
                })
            }
            TokenKind::LeftBrace => {
                let start = span.start;
                let mut statements = Vec::new();

                while !self.check(&TokenKind::RightBrace) && self.peek().is_some() {
                    statements.push(self.parse_statement()?);
                }

                self.expect(TokenKind::RightBrace)?;
                let span = self.span_from(start);
                Ok(Expr::Block { statements, span })
            }
            TokenKind::ThreadIdx => {
                let start = span.start;
                let dim = if self.match_token(&TokenKind::Dot) {
                    let tok = self.advance()?;
                    let tok_span = tok.span.clone();
                    Some(self.get_string_from_span(&tok_span))
                } else {
                    None
                };
                let span = self.span_from(start);
                Ok(Expr::ThreadIdx { dim, span })
            }
            TokenKind::BlockIdx => {
                let start = span.start;
                let dim = if self.match_token(&TokenKind::Dot) {
                    let tok = self.advance()?;
                    let tok_span = tok.span.clone();
                    Some(self.get_string_from_span(&tok_span))
                } else {
                    None
                };
                let span = self.span_from(start);
                Ok(Expr::BlockIdx { dim, span })
            }
            TokenKind::BlockDim => {
                let start = span.start;
                let dim = if self.match_token(&TokenKind::Dot) {
                    let tok = self.advance()?;
                    let tok_span = tok.span.clone();
                    Some(self.get_string_from_span(&tok_span))
                } else {
                    None
                };
                let span = self.span_from(start);
                Ok(Expr::BlockDim { dim, span })
            }
            TokenKind::Tensor => {
                let start = span.start;
                self.expect(TokenKind::Less)?;
                let dtype = self.parse_type()?;
                let mut shape = Vec::new();

                if self.match_token(&TokenKind::Comma) {
                    loop {
                        let tok = self.advance()?;
                        let tok_span = tok.span.clone();
                        let dim_expr = match &tok.kind {
                            TokenKind::Identifier(_) => {
                                let name = self.get_string_from_span(&tok_span);
                                Expr::Ident(name, tok_span)
                            }
                            TokenKind::IntLiteral(n) => Expr::IntLiteral(*n, tok_span),
                            _ => {
                                return Err(FlareError::UnexpectedToken(format!(
                                    "Expected dimension in tensor initialization, found {:?}",
                                    tok.kind
                                )))
                            }
                        };
                        shape.push(dim_expr);

                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                }

                self.expect(TokenKind::Greater)?;
                let span = self.span_from(start);
                Ok(Expr::TensorInit { dtype, shape, span })
            }
            _ => Err(FlareError::UnexpectedToken(format!(
                "unexpected token in expression: {:?}",
                token.kind
            ))),
        }
    }

    pub(crate) fn parse_block_expr(&mut self) -> Result<Expr<'src>, FlareError> {
        let start = self.expect(TokenKind::LeftBrace)?.span.start;
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) && self.peek().is_some() {
            statements.push(self.parse_statement()?);
        }

        self.expect(TokenKind::RightBrace)?;
        let span = self.span_from(start);
        Ok(Expr::Block { statements, span })
    }
}
