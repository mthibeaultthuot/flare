use core::{fmt, panic::PanicMessage};
use std::{iter::Empty, ops::Range};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, LoweringError>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LoweringError {
    #[error("invalid kernel configuration at {span:?}: {message}")]
    InvalidKernel { message: String, span: Range<usize> },

    #[error("format error {message}")]
    FormatError { message: String },
}

impl LoweringError {
    pub fn span(&self) -> &Range<usize> {
        static EMPTY: Range<usize> = 0..0;
        match self {
            LoweringError::InvalidKernel { span, .. } => span,
            LoweringError::FormatError { .. } => &EMPTY,
        }
    }

    pub fn lowering_error(message: impl Into<String>, span: Range<usize>) -> Self {
        LoweringError::InvalidKernel {
            message: message.into(),
            span,
        }
    }

    pub fn fmt_error(message: impl Into<String>) -> Self {
        LoweringError::FormatError {
            message: message.into(),
        }
    }
}

impl From<fmt::Error> for LoweringError {
    fn from(err: fmt::Error) -> Self {
        LoweringError::fmt_error(err.to_string())
    }
}
