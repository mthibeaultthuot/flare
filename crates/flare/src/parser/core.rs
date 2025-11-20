use crate::lexer::token::{Token, TokenKind};
use crate::{FlareError, Lexer};
use flare_ir::hir::*;
use std::ops::Range;

pub struct Parser<'src> {
    source: &'src str,
    pub tokens: Vec<Token<'src>>,
    pub current: usize,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Result<Self, FlareError> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();

        loop {
            match lexer.peek() {
                Some(Ok(token)) => {
                    if token.kind != TokenKind::Newline {
                        tokens.push(token);
                    }
                }
                Some(Err(e)) => return Err(e),
                None => break,
            }
        }

        Ok(Self {
            source,
            tokens,
            current: 0,
        })
    }

    pub fn parse(&mut self) -> Result<Program<'src>, FlareError> {
        self.parse_program()
    }

    pub(crate) fn peek(&self) -> Option<&Token<'src>> {
        self.tokens.get(self.current)
    }

    pub(crate) fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|t| &t.kind)
    }

    pub(crate) fn advance(&mut self) -> Result<&Token<'src>, FlareError> {
        if self.current >= self.tokens.len() {
            return Err(FlareError::UnexpectedEof);
        }
        let token = &self.tokens[self.current];
        self.current += 1;
        Ok(token)
    }

    pub(crate) fn expect(&mut self, expected: TokenKind) -> Result<&Token<'src>, FlareError> {
        let token = self.advance()?;
        if std::mem::discriminant(&token.kind) == std::mem::discriminant(&expected) {
            Ok(token)
        } else {
            Err(FlareError::UnexpectedToken(format!(
                "expected {:?}, found {:?} at {:?}",
                expected, token.kind, token.span
            )))
        }
    }

    pub(crate) fn check(&self, kind: &TokenKind) -> bool {
        if let Some(token) = self.peek() {
            std::mem::discriminant(&token.kind) == std::mem::discriminant(kind)
        } else {
            false
        }
    }

    pub(crate) fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.current += 1;
            true
        } else {
            false
        }
    }

    pub(crate) fn get_string_from_span(&self, span: &Range<usize>) -> &'src str {
        &self.source[span.start..span.end]
    }

    pub(crate) fn span_from(&self, start: usize) -> Range<usize> {
        let end = if self.current > 0 {
            self.tokens[self.current - 1].span.end
        } else {
            0
        };
        start..end
    }

    pub(crate) fn parse_type(&mut self) -> Result<Type<'src>, FlareError> {
        let token = self.advance()?;

        let base_type = match &token.kind {
            TokenKind::I32 => Type::I32,
            TokenKind::I64 => Type::I64,
            TokenKind::U32 => Type::U32,
            TokenKind::U64 => Type::U64,
            TokenKind::F32 => Type::F32,
            TokenKind::F64 => Type::F64,
            TokenKind::Bool => Type::Bool,
            TokenKind::Identifier(_) => {
                let span = token.span.clone();
                let name = self.get_string_from_span(&span);
                Type::Named(name)
            }
            TokenKind::Tensor => {
                self.expect(TokenKind::Less)?;
                let dtype = Box::new(self.parse_type()?);
                let mut shape = Vec::new();

                if self.match_token(&TokenKind::Comma) {
                    self.expect(TokenKind::LeftBracket)?;

                    if !self.check(&TokenKind::RightBracket) {
                        loop {
                            let tok = self.advance()?;
                            let tok_span = tok.span.clone();
                            if let TokenKind::Identifier(_) | TokenKind::IntLiteral(_) = &tok.kind {
                                shape.push(self.get_string_from_span(&tok_span));
                            } else {
                                return Err(FlareError::UnexpectedToken(format!(
                                    "expected dimension in tensor type, found {:?}",
                                    tok.kind
                                )));
                            }

                            if !self.match_token(&TokenKind::Comma) {
                                break;
                            }
                        }
                    }

                    self.expect(TokenKind::RightBracket)?;
                }

                self.expect(TokenKind::Greater)?;
                Type::Tensor { dtype, shape }
            }
            TokenKind::Matrix => {
                self.expect(TokenKind::Less)?;
                let dtype = Box::new(self.parse_type()?);
                let mut rows = None;
                let mut cols = None;

                if self.match_token(&TokenKind::Comma) {
                    let tok = self.advance()?;
                    let tok_span = tok.span.clone();
                    rows = Some(self.get_string_from_span(&tok_span));

                    if self.match_token(&TokenKind::Comma) {
                        let tok = self.advance()?;
                        let tok_span = tok.span.clone();
                        cols = Some(self.get_string_from_span(&tok_span));
                    }
                }

                self.expect(TokenKind::Greater)?;
                Type::Matrix { dtype, rows, cols }
            }
            TokenKind::Vector => {
                self.expect(TokenKind::Less)?;
                let dtype = Box::new(self.parse_type()?);
                let mut len = None;

                if self.match_token(&TokenKind::Comma) {
                    let tok = self.advance()?;
                    let tok_span = tok.span.clone();
                    len = Some(self.get_string_from_span(&tok_span));
                }

                self.expect(TokenKind::Greater)?;
                Type::Vector { dtype, len }
            }
            TokenKind::Star => {
                let inner = Box::new(self.parse_type()?);
                Type::Ptr(inner)
            }
            _ => {
                return Err(FlareError::UnexpectedToken(format!(
                    "expected type, found {:?}",
                    token.kind
                )))
            }
        };

        if self.check(&TokenKind::LeftBracket) {
            self.advance()?;

            let size = if let Some(TokenKind::IntLiteral(n)) = self.peek_kind() {
                let n = *n;
                self.advance()?;
                Some(n as usize)
            } else {
                None
            };
            self.expect(TokenKind::RightBracket)?;
            return Ok(Type::Array {
                dtype: Box::new(base_type),
                size,
            });
        }

        Ok(base_type)
    }

    pub(crate) fn parse_schedule(&mut self) -> Result<ScheduleBlock<'src>, FlareError> {
        let start = self.expect(TokenKind::Schedule)?.span.start;

        let target = if !self.check(&TokenKind::LeftBrace) {
            let name_token = self.expect(TokenKind::Identifier(String::new()))?;
            let span = name_token.span.clone();
            Some(self.get_string_from_span(&span))
        } else {
            None
        };

        self.expect(TokenKind::LeftBrace)?;
        let mut directives = Vec::new();

        while !self.check(&TokenKind::RightBrace) && self.peek().is_some() {
            if let Some(token) = self.peek() {
                match &token.kind {
                    TokenKind::Identifier(s) if s == "tile" => {
                        self.advance()?;
                        self.expect(TokenKind::LeftParen)?;
                        let x = if let TokenKind::IntLiteral(n) = self.advance()?.kind {
                            n
                        } else {
                            return Err(FlareError::UnexpectedToken(
                                "expected integer for tile x".to_string(),
                            ));
                        };

                        let y = if self.match_token(&TokenKind::Comma) {
                            if let TokenKind::IntLiteral(n) = self.advance()?.kind {
                                Some(n)
                            } else {
                                return Err(FlareError::UnexpectedToken(
                                    "expected integer for tile y".to_string(),
                                ));
                            }
                        } else {
                            None
                        };

                        let z = if y.is_some() && self.match_token(&TokenKind::Comma) {
                            if let TokenKind::IntLiteral(n) = self.advance()?.kind {
                                Some(n)
                            } else {
                                return Err(FlareError::UnexpectedToken(
                                    "expected integer for tile z".to_string(),
                                ));
                            }
                        } else {
                            None
                        };

                        self.expect(TokenKind::RightParen)?;
                        self.match_token(&TokenKind::Semicolon);
                        directives.push(ScheduleDirective::Tile { x, y, z });
                    }
                    TokenKind::Identifier(s) if s == "vectorize" => {
                        self.advance()?;
                        self.expect(TokenKind::LeftParen)?;
                        let n = if let TokenKind::IntLiteral(n) = self.advance()?.kind {
                            n
                        } else {
                            return Err(FlareError::UnexpectedToken(
                                "expected integer for vectorize".to_string(),
                            ));
                        };
                        self.expect(TokenKind::RightParen)?;
                        self.match_token(&TokenKind::Semicolon);
                        directives.push(ScheduleDirective::Vectorize(n));
                    }
                    TokenKind::Identifier(s) if s == "unroll" => {
                        self.advance()?;
                        self.expect(TokenKind::LeftParen)?;
                        let n = if let TokenKind::IntLiteral(n) = self.advance()?.kind {
                            n
                        } else {
                            return Err(FlareError::UnexpectedToken(
                                "expected integer for unroll".to_string(),
                            ));
                        };
                        self.expect(TokenKind::RightParen)?;
                        self.match_token(&TokenKind::Semicolon);
                        directives.push(ScheduleDirective::Unroll(n));
                    }
                    TokenKind::Identifier(s) if s == "threads" => {
                        self.advance()?;
                        self.expect(TokenKind::LeftParen)?;
                        let x = if let TokenKind::IntLiteral(n) = self.advance()?.kind {
                            n
                        } else {
                            return Err(FlareError::UnexpectedToken(
                                "expected integer for threads x".to_string(),
                            ));
                        };

                        let y = if self.match_token(&TokenKind::Comma) {
                            if let TokenKind::IntLiteral(n) = self.advance()?.kind {
                                Some(n)
                            } else {
                                return Err(FlareError::UnexpectedToken(
                                    "expected integer for threads y".to_string(),
                                ));
                            }
                        } else {
                            None
                        };

                        self.expect(TokenKind::RightParen)?;
                        self.match_token(&TokenKind::Semicolon);
                        directives.push(ScheduleDirective::Threads { x, y });
                    }
                    TokenKind::Memory => {
                        self.advance()?;
                        self.expect(TokenKind::LeftParen)?;
                        let var_token = self.expect(TokenKind::Identifier(String::new()))?;
                        let var_span = var_token.span.clone();
                        let var = self.get_string_from_span(&var_span);
                        self.expect(TokenKind::Comma)?;

                        let location_token = self.advance()?;
                        let location_span = location_token.span.clone();
                        let location = match &location_token.kind {
                            TokenKind::Identifier(s) if s == "shared" => MemoryLocation::Shared,
                            TokenKind::Identifier(s) if s == "global" => MemoryLocation::Global,
                            TokenKind::Identifier(s) if s == "local" => MemoryLocation::Local,
                            TokenKind::Identifier(s) if s == "constant" => MemoryLocation::Constant,
                            TokenKind::Persistent => MemoryLocation::Persistent,
                            TokenKind::Temporary => MemoryLocation::Temporary,
                            TokenKind::Streaming => MemoryLocation::Streaming,
                            TokenKind::Identifier(_) => {
                                MemoryLocation::Named(self.get_string_from_span(&location_span))
                            }
                            _ => {
                                return Err(FlareError::UnexpectedToken(
                                    "expected memory location".to_string(),
                                ))
                            }
                        };

                        self.expect(TokenKind::RightParen)?;
                        self.match_token(&TokenKind::Semicolon);
                        directives.push(ScheduleDirective::Memory { var, location });
                    }
                    TokenKind::Stream => {
                        self.advance()?;
                        self.expect(TokenKind::LeftParen)?;
                        let name_token = self.expect(TokenKind::Identifier(String::new()))?;
                        let name_span = name_token.span.clone();
                        let name = self.get_string_from_span(&name_span);
                        self.expect(TokenKind::RightParen)?;
                        self.match_token(&TokenKind::Semicolon);
                        directives.push(ScheduleDirective::Stream(name));
                    }
                    TokenKind::Pipeline => {
                        self.advance()?;
                        let depth = if self.match_token(&TokenKind::LeftParen) {
                            let d = if let TokenKind::IntLiteral(n) = self.advance()?.kind {
                                Some(n)
                            } else {
                                None
                            };
                            self.expect(TokenKind::RightParen)?;
                            d
                        } else {
                            None
                        };
                        self.match_token(&TokenKind::Semicolon);
                        directives.push(ScheduleDirective::Pipeline { depth });
                    }
                    TokenKind::Parallel => {
                        self.advance()?;
                        self.match_token(&TokenKind::Semicolon);
                        directives.push(ScheduleDirective::Parallel);
                    }
                    _ => {
                        return Err(FlareError::UnexpectedToken(format!(
                            "unknown schedule directive: {:?}",
                            token.kind
                        )))
                    }
                }
            }
        }

        self.expect(TokenKind::RightBrace)?;
        let span = self.span_from(start);
        Ok(ScheduleBlock {
            target,
            directives,
            span,
        })
    }

    pub(crate) fn parse_fusion(&mut self) -> Result<FusionBlock<'src>, FlareError> {
        let start = self.expect(TokenKind::Fuse)?.span.start;

        let mut targets = Vec::new();
        loop {
            let name_token = self.expect(TokenKind::Identifier(String::new()))?;
            let name_span = name_token.span.clone();
            targets.push(self.get_string_from_span(&name_span));

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        let strategy = if self.match_token(&TokenKind::Colon) {
            if let Some(token) = self.peek() {
                match &token.kind {
                    TokenKind::Identifier(s) if s == "elementwise" => {
                        self.advance()?;
                        Some(FusionStrategy::Elementwise)
                    }
                    TokenKind::Inline => {
                        self.advance()?;
                        Some(FusionStrategy::Inline)
                    }
                    TokenKind::Auto => {
                        self.advance()?;
                        Some(FusionStrategy::Auto)
                    }
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        let mut barriers = Vec::new();
        if self.match_token(&TokenKind::Where) {
            self.expect(TokenKind::Identifier(String::from("barriers")))?;
            self.expect(TokenKind::Assign)?;
            self.expect(TokenKind::LeftBracket)?;

            if !self.check(&TokenKind::RightBracket) {
                loop {
                    let barrier_token = self.expect(TokenKind::Identifier(String::new()))?;
                    let barrier_span = barrier_token.span.clone();
                    barriers.push(self.get_string_from_span(&barrier_span));

                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
            }

            self.expect(TokenKind::RightBracket)?;
        }

        self.match_token(&TokenKind::Semicolon);

        let span = self.span_from(start);
        Ok(FusionBlock {
            targets,
            strategy,
            barriers,
            span,
        })
    }

    pub(crate) fn parse_program(&mut self) -> Result<Program<'src>, FlareError> {
        let start = 0;
        let mut items = Vec::new();

        while self.peek().is_some() {
            let mut attributes = Vec::new();
            while self.check(&TokenKind::At) {
                attributes.push(self.parse_attribute()?);
            }

            if let Some(token) = self.peek() {
                match &token.kind {
                    TokenKind::Kernel => {
                        let mut kernel = self.parse_kernel()?;
                        kernel.attributes = attributes;
                        items.push(Stmt::Kernel(kernel));
                    }
                    TokenKind::Fuse => {
                        let fusion = self.parse_fusion()?;
                        items.push(Stmt::Fusion(fusion));
                    }
                    TokenKind::Schedule => {
                        let schedule = self.parse_schedule()?;
                        items.push(Stmt::Schedule(schedule));
                    }
                    TokenKind::Fn => {
                        items.push(self.parse_statement()?);
                    }
                    TokenKind::Type => {
                        items.push(self.parse_statement()?);
                    }
                    TokenKind::Let => {
                        items.push(self.parse_statement()?);
                    }
                    _ => {
                        return Err(FlareError::UnexpectedToken(format!(
                            "Expected top-level item, found {:?}",
                            token.kind
                        )))
                    }
                }
            }
        }

        let span = self.span_from(start);
        Ok(Program { items, span })
    }
}
