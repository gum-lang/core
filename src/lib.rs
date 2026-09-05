//! Gum Markup Language — Core Engine
//!
//! This crate provides the canonical reference implementation for parsing,
//! validating, and formatting Gum markup documents.

mod ast;
mod error;
mod formatter;
mod parser;
mod validator;

#[cfg(target_arch = "wasm32")]
mod wasm;

pub use ast::*;
pub use error::{GumError, Result};
pub use formatter::format;
pub use parser::parse;
pub use validator::validate;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

/// Parse a Gum document from a string.
///
/// Returns the parsed AST or an error if the input is malformed.
///
/// # Example
/// ```
/// use gum_core::parse;
///
/// let input = r#"name = "Gum""#;
/// let doc = parse(input).expect("valid document");
/// assert_eq!(doc.assignments().len(), 1);
/// ```
pub fn parse_document(input: &str) -> Result<Document> {
    parse(input)
}

/// Validate a parsed document for semantic correctness.
///
/// Returns a list of validation errors (empty if valid).
pub fn validate_document(doc: &Document) -> Vec<GumError> {
    validate(doc)
}

/// Format a document back to a normalized string.
pub fn format_document(doc: &Document) -> String {
    format(doc)
}
