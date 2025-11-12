use core::{fmt, panic::PanicMessage};
use std::ops::Range;
use thiserror::Error;

use crate::error;

pub type Result<T> = std::result::Result<T, CodegenError>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CodegenError {
    #[error("unsupported type for Metal backend at {span:?}: {message}")]
    UnsupportedType { message: String, span: Range<usize> },

    #[error("feature not supported in Metal at {span:?}: {feature}")]
    UnsupportedFeature {
        feature: String,
        span: Range<usize>,
        suggestion: Option<String>,
    },

    #[error("invalid kernel configuration at {span:?}: {message}")]
    InvalidKernelConfig { message: String, span: Range<usize> },

    #[error("invalid schedule directive at {span:?}: {message}")]
    InvalidScheduleDirective { message: String, span: Range<usize> },

    #[error("invalid memory configuration at {span:?}: {message}")]
    InvalidMemoryConfig { message: String, span: Range<usize> },

    #[error("failed to generate expression at {span:?}: {message}")]
    ExpressionError { message: String, span: Range<usize> },

    #[error("failed to generate statement at {span:?}: {message}")]
    StatementError { message: String, span: Range<usize> },

    #[error("invalid identifier '{name}' at {span:?}: {reason}")]
    InvalidIdentifier {
        name: String,
        reason: String,
        span: Range<usize>,
    },

    #[error("resource limit exceeded at {span:?}: {message}")]
    ResourceLimitExceeded { message: String, span: Range<usize> },

    #[error("internal compiler error at {span:?}: {message}")]
    InternalError { message: String, span: Range<usize> },

    #[error("format error : {message}")]
    FormatError { message: String },
}

impl CodegenError {
    pub fn span(&self) -> &Range<usize> {
        static EMPTY: Range<usize> = 0..0;
        match self {
            CodegenError::UnsupportedType { span, .. }
            | CodegenError::UnsupportedFeature { span, .. }
            | CodegenError::InvalidKernelConfig { span, .. }
            | CodegenError::InvalidScheduleDirective { span, .. }
            | CodegenError::InvalidMemoryConfig { span, .. }
            | CodegenError::ExpressionError { span, .. }
            | CodegenError::StatementError { span, .. }
            | CodegenError::InvalidIdentifier { span, .. }
            | CodegenError::ResourceLimitExceeded { span, .. }
            | CodegenError::InternalError { span, .. } => span,
            CodegenError::FormatError { .. } => &EMPTY,
        }
    }

    pub fn unsupported_type(message: impl Into<String>, span: Range<usize>) -> Self {
        CodegenError::UnsupportedType {
            message: message.into(),
            span,
        }
    }

    pub fn unsupported_feature(
        feature: impl Into<String>,
        span: Range<usize>,
        suggestion: Option<String>,
    ) -> Self {
        CodegenError::UnsupportedFeature {
            feature: feature.into(),
            span,
            suggestion,
        }
    }

    pub fn invalid_kernel_config(message: impl Into<String>, span: Range<usize>) -> Self {
        CodegenError::InvalidKernelConfig {
            message: message.into(),
            span,
        }
    }

    pub fn invalid_schedule_directive(message: impl Into<String>, span: Range<usize>) -> Self {
        CodegenError::InvalidScheduleDirective {
            message: message.into(),
            span,
        }
    }

    pub fn invalid_memory_config(message: impl Into<String>, span: Range<usize>) -> Self {
        CodegenError::InvalidMemoryConfig {
            message: message.into(),
            span,
        }
    }

    pub fn expression_error(message: impl Into<String>, span: Range<usize>) -> Self {
        CodegenError::ExpressionError {
            message: message.into(),
            span,
        }
    }

    pub fn statement_error(message: impl Into<String>, span: Range<usize>) -> Self {
        CodegenError::StatementError {
            message: message.into(),
            span,
        }
    }

    pub fn invalid_identifier(
        name: impl Into<String>,
        reason: impl Into<String>,
        span: Range<usize>,
    ) -> Self {
        CodegenError::InvalidIdentifier {
            name: name.into(),
            reason: reason.into(),
            span,
        }
    }

    pub fn resource_limit_exceeded(message: impl Into<String>, span: Range<usize>) -> Self {
        CodegenError::ResourceLimitExceeded {
            message: message.into(),
            span,
        }
    }

    pub fn internal_error(message: impl Into<String>, span: Range<usize>) -> Self {
        CodegenError::InternalError {
            message: message.into(),
            span,
        }
    }

    pub fn fmt_error(message: impl Into<String>) -> Self {
        CodegenError::FormatError {
            message: message.into(),
        }
    }
}

impl From<fmt::Error> for CodegenError {
    fn from(err: fmt::Error) -> Self {
        CodegenError::fmt_error(err.to_string())
    }
}
