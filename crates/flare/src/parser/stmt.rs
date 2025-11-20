use super::core::Parser;
use crate::lexer::token::TokenKind;
use crate::FlareError;
use flare_ir::hir::*;

impl<'src> Parser<'src> {
    pub(crate) fn parse_statement(&mut self) -> Result<Stmt<'src>, FlareError> {
        if let Some(token) = self.peek() {
            match &token.kind {
                TokenKind::Let => self.parse_let_statement(),
                TokenKind::Var => self.parse_var_statement(),
                TokenKind::Const => self.parse_const_statement(),
                TokenKind::If => self.parse_if_statement(),
                TokenKind::While => self.parse_while_statement(),
                TokenKind::For => self.parse_for_statement(),
                TokenKind::Return => self.parse_return_statement(),
                TokenKind::LeftBrace => self.parse_block_statement(),
                TokenKind::SyncThreads => self.parse_sync_threads(),
                TokenKind::LoadShared => self.parse_load_shared(),
                TokenKind::Type => self.parse_type_def(),
                TokenKind::Fn => self.parse_function(),
                _ => {
                    let expr = self.parse_expression()?;
                    if self.match_token(&TokenKind::Semicolon) {}
                    Ok(Stmt::Expr(expr))
                }
            }
        } else {
            Err(FlareError::UnexpectedEof)
        }
    }

    fn parse_let_statement(&mut self) -> Result<Stmt<'src>, FlareError> {
        let start = self.expect(TokenKind::Let)?.span.start;
        let name_token = self.expect(TokenKind::Identifier(String::new()))?;
        let name_token_span = name_token.span.clone();
        let name = self.get_string_from_span(&name_token_span);

        let ty = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(TokenKind::Assign)?;
        let value = self.parse_expression()?;
        self.match_token(&TokenKind::Semicolon);

        let span = self.span_from(start);
        Ok(Stmt::Let {
            name,
            ty,
            value,
            span,
        })
    }

    fn parse_var_statement(&mut self) -> Result<Stmt<'src>, FlareError> {
        let start = self.expect(TokenKind::Var)?.span.start;
        let name_token = self.expect(TokenKind::Identifier(String::new()))?;
        let name_token_span = name_token.span.clone();
        let name = self.get_string_from_span(&name_token_span);

        let ty = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let value = if self.match_token(&TokenKind::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.match_token(&TokenKind::Semicolon);

        let span = self.span_from(start);
        Ok(Stmt::Var {
            name,
            ty,
            value,
            span,
        })
    }

    fn parse_const_statement(&mut self) -> Result<Stmt<'src>, FlareError> {
        let start = self.expect(TokenKind::Const)?.span.start;
        let name_token = self.expect(TokenKind::Identifier(String::new()))?;
        let name_token_span = name_token.span.clone();
        let name = self.get_string_from_span(&name_token_span);

        let ty = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(TokenKind::Assign)?;
        let value = self.parse_expression()?;
        self.match_token(&TokenKind::Semicolon);

        let span = self.span_from(start);
        Ok(Stmt::Const {
            name,
            ty,
            value,
            span,
        })
    }

    fn parse_if_statement(&mut self) -> Result<Stmt<'src>, FlareError> {
        let start = self.expect(TokenKind::If)?.span.start;
        let condition = self.parse_expression()?;
        let then_branch = Box::new(self.parse_statement()?);
        let else_branch = if self.match_token(&TokenKind::Else) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        let span = self.span_from(start);
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
            span,
        })
    }

    fn parse_while_statement(&mut self) -> Result<Stmt<'src>, FlareError> {
        let start = self.expect(TokenKind::While)?.span.start;
        let condition = self.parse_expression()?;
        let body = Box::new(self.parse_statement()?);

        let span = self.span_from(start);
        Ok(Stmt::While {
            condition,
            body,
            span,
        })
    }

    fn parse_for_statement(&mut self) -> Result<Stmt<'src>, FlareError> {
        let start = self.expect(TokenKind::For)?.span.start;
        let var_token = self.expect(TokenKind::Identifier(String::new()))?;
        let var_token_span = var_token.span.clone();
        let var = self.get_string_from_span(&var_token_span);
        self.expect(TokenKind::In)?;
        let iterator = self.parse_expression()?;
        let body = Box::new(self.parse_statement()?);

        let span = self.span_from(start);
        Ok(Stmt::For {
            var,
            iterator,
            body,
            span,
        })
    }

    fn parse_return_statement(&mut self) -> Result<Stmt<'src>, FlareError> {
        let start = self.expect(TokenKind::Return)?.span.start;
        let value = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.match_token(&TokenKind::Semicolon);

        let span = self.span_from(start);
        Ok(Stmt::Return { value, span })
    }

    fn parse_block_statement(&mut self) -> Result<Stmt<'src>, FlareError> {
        let start = self.expect(TokenKind::LeftBrace)?.span.start;
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) && self.peek().is_some() {
            statements.push(self.parse_statement()?);
        }

        self.expect(TokenKind::RightBrace)?;
        let span = self.span_from(start);
        Ok(Stmt::Block { statements, span })
    }

    fn parse_sync_threads(&mut self) -> Result<Stmt<'src>, FlareError> {
        let start = self.expect(TokenKind::SyncThreads)?.span.start;
        self.expect(TokenKind::LeftParen)?;
        self.expect(TokenKind::RightParen)?;
        self.match_token(&TokenKind::Semicolon);

        let span = self.span_from(start);
        Ok(Stmt::SyncThreads { span })
    }

    fn parse_load_shared(&mut self) -> Result<Stmt<'src>, FlareError> {
        let start = self.expect(TokenKind::LoadShared)?.span.start;
        self.expect(TokenKind::LeftParen)?;
        let dest_token = self.expect(TokenKind::Identifier(String::new()))?;
        let dest_token_span = dest_token.span.clone();
        let dest = self.get_string_from_span(&dest_token_span);
        self.expect(TokenKind::Comma)?;
        let src = self.parse_expression()?;
        self.expect(TokenKind::RightParen)?;
        self.match_token(&TokenKind::Semicolon);

        let span = self.span_from(start);
        Ok(Stmt::LoadShared { dest, src, span })
    }

    fn parse_type_def(&mut self) -> Result<Stmt<'src>, FlareError> {
        let start = self.expect(TokenKind::Type)?.span.start;
        let name_token = self.expect(TokenKind::Identifier(String::new()))?;
        let name_token_span = name_token.span.clone();
        let name = self.get_string_from_span(&name_token_span);
        self.expect(TokenKind::Assign)?;
        let ty = self.parse_type()?;
        self.match_token(&TokenKind::Semicolon);

        let span = self.span_from(start);
        Ok(Stmt::TypeDef { name, ty, span })
    }

    fn parse_function(&mut self) -> Result<Stmt<'src>, FlareError> {
        let start = self.expect(TokenKind::Fn)?.span.start;
        let name_token = self.expect(TokenKind::Identifier(String::new()))?;
        let name_token_span = name_token.span.clone();
        let name = self.get_string_from_span(&name_token_span);

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
                    ty: param_type,
                    span: param_span,
                });

                if !self.match_token(&TokenKind::Comma) {
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

        let body = Box::new(self.parse_block_expr()?);

        let span = self.span_from(start);
        Ok(Stmt::Function {
            name,
            params,
            return_type,
            body,
            span,
        })
    }
}
