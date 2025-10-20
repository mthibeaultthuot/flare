use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum FlareError {
    #[error("Unexpected character '{ch} at position {pos}'")]
    UnexpectedChar { ch: char, pos: usize },
    #[error("invalid token at {span:?}: {error}")]
    InvalidToken {
        error: String,
        span: std::ops::Range<usize>,
    },
    #[error("UnexpectedEof")]
    UnexpectedEof,

    #[error("UnexpectedToken {0}")]
    UnexpectedToken(String),
}
