//! Error types for the Gum parser.

use serde::Serialize;
use thiserror::Error;

/// Errors that can occur during parsing or validation.
#[derive(Error, Debug, Clone, PartialEq, Serialize)]
pub enum GumError {
    /// A parsing error with location information.
    #[error("Parse error at line {line}, column {column}: {message}")]
    ParseError {
        /// The line number (1-indexed).
        line: usize,
        /// The column number (1-indexed).
        column: usize,
        /// A description of the error.
        message: String,
    },

    /// A validation error (semantic issue).
    #[error("Validation error: {message}")]
    ValidationError {
        /// A description of the error.
        message: String,
    },

    /// Reserved keyword used as bare key.
    #[error("Reserved keyword '{keyword}' cannot be used as a bare key")]
    ReservedKeyword {
        /// The reserved keyword that was used.
        keyword: String,
    },

    /// Invalid escape sequence in string.
    #[error("Invalid escape sequence: {sequence}")]
    InvalidEscape {
        /// The invalid escape sequence.
        sequence: String,
    },

    /// Invalid number format.
    #[error("Invalid number format: {value}")]
    InvalidNumber {
        /// The invalid number string.
        value: String,
    },

    /// Unexpected end of input.
    #[error("Unexpected end of input")]
    UnexpectedEof,

    /// Unexpected character.
    #[error("Unexpected character '{character}' at line {line}, column {column}")]
    UnexpectedCharacter {
        /// The unexpected character.
        character: char,
        /// The line number (1-indexed).
        line: usize,
        /// The column number (1-indexed).
        column: usize,
    },
}

/// Result type for Gum operations.
pub type Result<T> = std::result::Result<T, GumError>;

impl GumError {
    /// Create a parse error at the given location.
    pub fn parse(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::ParseError {
            line,
            column,
            message: message.into(),
        }
    }

    /// Create a validation error.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
        }
    }
}
