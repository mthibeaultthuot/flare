use crate::lexer::token::Token;
use crate::{error::FlareError, lexer::token::TokenKind};
use logos::{Lexer as LogosLexer, Logos};

pub struct Lexer<'src> {
    pub input: &'src str,
    pub inner: LogosLexer<'src, TokenKind>,
    current: usize,
    pub peeked: Option<Result<Token<'src>, FlareError>>,
}

impl<'src> Lexer<'src> {
    pub fn new(input: &'src str) -> Self {
        Self {
            input,
            inner: TokenKind::lexer(input),
            current: 0,
            peeked: None,
        }
    }

    pub fn peek(&mut self) -> Option<Result<Token<'src>, FlareError>> {
        let new_peek = Some(Ok(Token::new(
            self.inner
                .next()
                .ok_or_else(|| FlareError::UnexpectedToken(String::from(self.inner.slice())))
                .ok()?
                .unwrap(),
            self.current,
            self.inner.slice(),
            self.inner.span(),
        )));
        new_peek
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let source = "kernel matmul<T>(){  }";
        let mut lexer = Lexer::new(source);
        assert_eq!(
            lexer.peek().unwrap().unwrap(),
            Token {
                kind: TokenKind::Kernel,
                idx: 0,
                text: "kernel",
                span: 0..6
            }
        );
        assert_eq!(
            lexer.peek().unwrap().unwrap(),
            Token {
                kind: TokenKind::Identifier("matmul".to_string()),
                idx: 0,
                text: "matmul",
                span: 7..13
            }
        );
    }
}
