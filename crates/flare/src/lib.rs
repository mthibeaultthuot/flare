pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;

pub use crate::lexer::token::Token;
pub use ast::Program;
pub use error::FlareError;
pub use lexer::core::Lexer;
pub use parser::core::Parser;

pub struct Flare;

impl Flare {
    pub fn compile_from_string(source: &str) -> Result<Program<'_>, FlareError> {
        let mut parser = Parser::new(source)?;
        let program = parser.parse()?;
        Ok(program)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matmul_naive_loops() {
        let source = r#"
            kernel matmul_naive(A: Tensor<f32, [M, K]>, B: Tensor<f32, [K, N]>) -> Tensor<f32, [M, N]> {
                grid: [M, N]
                block: [1]

                compute {
                    let row = block_idx.y
                    let col = block_idx.x
                    var sum: f32 = 0.0

                    for k in 0..K {
                        sum = sum + A[row, k] * B[k, col]
                    }

                    output[row, col] = sum
                }
            }
        "#;

        let result = Flare::compile_from_string(source);
        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn simple_parsing() {
        let source = r#"
            kernel simple_allocation() {
                let i = 67;
            }
        "#;
        let mut lexer = Lexer::new(source);
        if let Some(token) = lexer.peek() {
            println!("{:?}", token);
        }
        let ast = Flare::compile_from_string(source);

        println!("result {:?}", lexer.inner);
        println!("result {:?}", ast);
    }
}
