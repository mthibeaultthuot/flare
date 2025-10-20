use crate::{FlareError, Lexer};

pub struct Parser<'src> {
    lexer: Lexer<'src>,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            lexer: Lexer::new(source),
        }
    }

    pub fn parse(&mut self) -> Result<(), FlareError> {
        Ok(())
    }
}
