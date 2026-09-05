//! Integration tests for the Gum parser.
//!
//! These tests verify that the parser correctly handles all language features
//! defined in the ABNF grammar, using golden files for test cases.

use gum_core::{parse, validate};
use std::fs;
use std::path::Path;

/// Test that all valid .gum files parse successfully.
#[test]
fn test_valid_files() {
    let test_dir = Path::new("integration_tests/valid");
    if !test_dir.exists() {
        return;
    }

    for entry in fs::read_dir(test_dir).expect("Failed to read test directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("gum") {
            let content = fs::read_to_string(&path).expect("Failed to read test file");
            let file_name = path.file_name().unwrap().to_str().unwrap();

            match parse(&content) {
                Ok(doc) => {
                    // Also validate the document
                    let errors = validate(&doc);
                    assert!(
                        errors.is_empty(),
                        "Validation errors in {}: {:?}",
                        file_name,
                        errors
                    );
                }
                Err(e) => {
                    panic!("Failed to parse valid file {}: {}", file_name, e);
                }
            }
        }
    }
}

/// Test that all invalid .gum files fail to parse or validate.
#[test]
fn test_invalid_files() {
    let test_dir = Path::new("integration_tests/invalid");
    if !test_dir.exists() {
        return;
    }

    for entry in fs::read_dir(test_dir).expect("Failed to read test directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("gum") {
            let content = fs::read_to_string(&path).expect("Failed to read test file");
            let file_name = path.file_name().unwrap().to_str().unwrap();

            // The file should either fail to parse or fail validation
            let result = parse(&content).and_then(|doc| {
                let errors = validate(&doc);
                if errors.is_empty() {
                    Ok(doc)
                } else {
                    Err(errors.into_iter().next().unwrap())
                }
            });

            assert!(
                result.is_err(),
                "Expected error in invalid file {}, but it parsed successfully",
                file_name
            );
        }
    }
}

/// Test roundtrip: parse -> format -> parse should yield identical ASTs.
#[test]
fn test_roundtrip() {
    let test_dir = Path::new("integration_tests/valid");
    if !test_dir.exists() {
        return;
    }

    for entry in fs::read_dir(test_dir).expect("Failed to read test directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("gum") {
            let content = fs::read_to_string(&path).expect("Failed to read test file");
            let file_name = path.file_name().unwrap().to_str().unwrap();

            let doc1 = parse(&content).expect("Failed to parse");
            let formatted = gum_core::format(&doc1);
            let doc2 = parse(&formatted).expect("Failed to parse formatted output");

            assert_eq!(
                doc1, doc2,
                "Roundtrip failed for {}: ASTs don't match",
                file_name
            );
        }
    }
}

/// Test specific language features.
#[test]
fn test_basic_assignment() {
    let input = r#"name = "Gum""#;
    let doc = parse(input).expect("Failed to parse");
    assert_eq!(doc.assignments().len(), 1);
}

#[test]
fn test_dotted_keys() {
    let input = r#"server.host = "localhost""#;
    let doc = parse(input).expect("Failed to parse");
    let assignments = doc.assignments();
    assert_eq!(assignments.len(), 1);
    assert!(matches!(assignments[0].key, gum_core::Key::Dotted(_)));
}

#[test]
fn test_all_value_types() {
    let input = r#"
string = "text"
number = 42
float = 3.14
bool_true = true
bool_false = false
null_val = null
table = { key = "value" }
list = [1, 2, 3]
"#;
    let doc = parse(input).expect("Failed to parse");
    assert_eq!(doc.assignments().len(), 8);
}

#[test]
fn test_reserved_keywords() {
    // Reserved keywords should fail as bare keys
    assert!(parse(r#"true = "value""#).is_err());
    assert!(parse(r#"false = "value""#).is_err());
    assert!(parse(r#"null = "value""#).is_err());

    // But should work as values
    assert!(parse(r#"key = true"#).is_ok());
    assert!(parse(r#"key = false"#).is_ok());
    assert!(parse(r#"key = null"#).is_ok());
}

#[test]
fn test_string_escapes() {
    let input = r#"escaped = "Tab:\t Newline:\n Quote:\"""#;
    let doc = parse(input).expect("Failed to parse");
    let assignments = doc.assignments();
    assert_eq!(assignments.len(), 1);
}

#[test]
fn test_multiline_strings() {
    let input = r#"
multiline = """
    Line 1
    Line 2
"""
"#;
    let doc = parse(input).expect("Failed to parse");
    assert_eq!(doc.assignments().len(), 1);
}

#[test]
fn test_comments() {
    let input = r#"
# Comment
name = "Gum" # Inline comment
# Another comment
version = 1
"#;
    let doc = parse(input).expect("Failed to parse");
    assert_eq!(doc.assignments().len(), 2);
}

#[test]
fn test_nested_tables() {
    let input = r#"
outer = {
    inner = {
        value = 42
    }
}
"#;
    let doc = parse(input).expect("Failed to parse");
    assert_eq!(doc.assignments().len(), 1);
}

#[test]
fn test_nested_lists() {
    let input = r#"nested = [[1, 2], [3, 4]]"#;
    let doc = parse(input).expect("Failed to parse");
    assert_eq!(doc.assignments().len(), 1);
}
