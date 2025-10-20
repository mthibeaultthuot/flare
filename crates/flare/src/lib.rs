pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;

pub use crate::lexer::token::Token;
pub use error::FlareError;
pub use lexer::core::Lexer;

pub struct Flare;

impl Flare {
    pub fn compile_from_string(source: &str) -> Result<(), FlareError> {
        let _lexer = Lexer::new(source);
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
