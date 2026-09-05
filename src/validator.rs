//! Validator for semantic correctness of Gum documents.

use std::collections::HashSet;

use crate::ast::*;
use crate::error::GumError;

/// Validate a document for semantic correctness.
///
/// Returns a list of validation errors (empty if valid).
pub fn validate(doc: &Document) -> Vec<GumError> {
    let mut errors = Vec::new();

    // Check for duplicate top-level keys
    let mut seen_keys = HashSet::new();
    for element in &doc.elements {
        if let DocumentElement::Assignment(assignment) = element {
            let key_str = assignment.key.as_str();
            if !seen_keys.insert(key_str.clone()) {
                errors.push(GumError::validation(format!(
                    "Duplicate key '{}' at top level",
                    key_str
                )));
            }
        }
    }

    // Validate each assignment recursively
    for element in &doc.elements {
        if let DocumentElement::Assignment(assignment) = element {
            validate_assignment(assignment, &mut errors);
        }
    }

    errors
}

/// Validate a single assignment.
fn validate_assignment(assignment: &Assignment, errors: &mut Vec<GumError>) {
    // Validate the key
    validate_key(&assignment.key, errors);

    // Validate the value
    validate_value(&assignment.value, errors);
}

/// Validate a key.
fn validate_key(key: &Key, errors: &mut Vec<GumError>) {
    match key {
        Key::Bare(s) => {
            // Check for reserved keywords
            if s == "true" || s == "false" || s == "null" {
                errors.push(GumError::ReservedKeyword { keyword: s.clone() });
            }
        }
        Key::Dotted(keys) => {
            // Validate each part of the dotted key
            for k in keys {
                validate_key(k, errors);
            }
        }
        Key::Quoted(_) => {
            // Quoted keys can be anything
        }
    }
}

/// Validate a value recursively.
fn validate_value(value: &Value, errors: &mut Vec<GumError>) {
    match value {
        Value::Table(table) => {
            // Check for duplicate keys in table
            let mut seen_keys = HashSet::new();
            for entry in &table.entries {
                let key_str = entry.key.as_str();
                if !seen_keys.insert(key_str.clone()) {
                    errors.push(GumError::validation(format!(
                        "Duplicate key '{}' in table",
                        key_str
                    )));
                }
                validate_assignment(entry, errors);
            }
        }
        Value::List(list) => {
            // Validate each element in the list
            for element in &list.elements {
                validate_value(element, errors);
            }
        }
        _ => {
            // Other value types don't need validation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_valid_document() {
        let input = r#"
name = "Gum"
version = 1
"#;
        let doc = parse(input).unwrap();
        let errors = validate(&doc);
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_duplicate_top_level_key() {
        let input = r#"
name = "first"
name = "second"
"#;
        let doc = parse(input).unwrap();
        let errors = validate(&doc);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0],
            GumError::ValidationError { ref message } if message.contains("Duplicate key")
        ));
    }

    #[test]
    fn test_duplicate_table_key() {
        let input = r#"
config = {
    name = "first"
    name = "second"
}
"#;
        let doc = parse(input).unwrap();
        let errors = validate(&doc);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0],
            GumError::ValidationError { ref message } if message.contains("Duplicate key")
        ));
    }
}
