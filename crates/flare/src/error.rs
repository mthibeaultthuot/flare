use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum FlareError {
    #[error("unexpected character '{ch} at position {pos}'")]
    UnexpectedChar { ch: char, pos: usize },
    #[error("invalid token at {span:?}: {error}")]
    InvalidToken {
        error: String,
        span: std::ops::Range<usize>,
    },
    #[error("unexpectedEof")]
    UnexpectedEof,

    #[error("unexpectedToken {0}")]
    UnexpectedToken(String),
}
