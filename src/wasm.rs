//! WebAssembly bindings for the Gum parser.
//!
//! This module provides JavaScript-friendly wrappers around the core parser
//! functionality for use in browser-based tools and playgrounds.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use serde_wasm_bindgen;

/// Parse a Gum document and return the AST as a JavaScript object.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn parse_to_ast(input: &str) -> Result<JsValue, JsValue> {
    match crate::parse(input) {
        Ok(doc) => serde_wasm_bindgen::to_value(&doc)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e))),
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

/// Parse a Gum document and return a formatted string.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn format_document(input: &str) -> Result<String, JsValue> {
    match crate::parse(input) {
        Ok(doc) => Ok(crate::format(&doc)),
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

/// Validate a Gum document and return a list of errors.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn validate_document(input: &str) -> Result<JsValue, JsValue> {
    match crate::parse(input) {
        Ok(doc) => {
            let errors = crate::validate(&doc);
            serde_wasm_bindgen::to_value(&errors)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
        }
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

/// Check if a Gum document is valid.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn is_valid(input: &str) -> bool {
    match crate::parse(input) {
        Ok(doc) => crate::validate(&doc).is_empty(),
        Err(_) => false,
    }
}
