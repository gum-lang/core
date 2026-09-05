//! Formatter for converting AST back to normalized Gum strings.

use crate::ast::*;

/// Format a document back to a normalized string.
pub fn format(doc: &Document) -> String {
    let mut output = String::new();

    for element in &doc.elements {
        match element {
            DocumentElement::Assignment(assignment) => {
                format_assignment(assignment, &mut output);
                output.push('\n');
            }
            DocumentElement::NonAssignment(non_assignment) => {
                output.push_str(&non_assignment.content);
            }
        }
    }

    output
}

/// Format a single assignment.
fn format_assignment(assignment: &Assignment, output: &mut String) {
    format_key(&assignment.key, output);
    output.push_str(" = ");
    format_value(&assignment.value, output);
}

/// Format a key.
fn format_key(key: &Key, output: &mut String) {
    match key {
        Key::Bare(s) => {
            output.push_str(s);
        }
        Key::Quoted(s) => {
            format_basic_string(s, output);
        }
        Key::Dotted(keys) => {
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    output.push('.');
                }
                format_key(k, output);
            }
        }
    }
}

/// Format a value.
fn format_value(value: &Value, output: &mut String) {
    match value {
        Value::String(s) => format_string(s, output),
        Value::Number(n) => format_number(n, output),
        Value::Boolean(b) => {
            output.push_str(if *b { "true" } else { "false" });
        }
        Value::Null => {
            output.push_str("null");
        }
        Value::Table(t) => format_table(t, output),
        Value::List(l) => format_list(l, output),
    }
}

/// Format a string value.
fn format_string(s: &StringValue, output: &mut String) {
    match s {
        StringValue::Basic(s) => {
            format_basic_string(s, output);
        }
        StringValue::MultilineBasic(s) => {
            format_multiline_basic_string(s, output);
        }
        StringValue::Literal(s) => {
            format_literal_string(s, output);
        }
        StringValue::MultilineLiteral(s) => {
            format_multiline_literal_string(s, output);
        }
    }
}

/// Format a basic string (double-quoted).
fn format_basic_string(s: &str, output: &mut String) {
    output.push('"');
    for ch in s.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\x08' => output.push_str("\\b"),
            '\x0C' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            c if c.is_control() => {
                output.push_str(&format!("\\u{:04X}", c as u32));
            }
            c => output.push(c),
        }
    }
    output.push('"');
}

/// Format a multiline basic string (triple double-quoted).
fn format_multiline_basic_string(s: &str, output: &mut String) {
    output.push_str("\"\"\"\n");
    for ch in s.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '\x08' => output.push_str("\\b"),
            '\x0C' => output.push_str("\\f"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            c if c.is_control() && c != '\n' => {
                output.push_str(&format!("\\u{:04X}", c as u32));
            }
            c => output.push(c),
        }
    }
    output.push_str("\"\"\"");
}

/// Format a literal string (single-quoted).
fn format_literal_string(s: &str, output: &mut String) {
    output.push('\'');
    output.push_str(s);
    output.push('\'');
}

/// Format a multiline literal string (triple single-quoted).
fn format_multiline_literal_string(s: &str, output: &mut String) {
    output.push_str("'''\n");
    output.push_str(s);
    output.push_str("'''");
}

/// Format a number value.
fn format_number(n: &NumberValue, output: &mut String) {
    match n {
        NumberValue::Integer(i) => {
            output.push_str(&i.to_string());
        }
        NumberValue::Float(f) => {
            // Format float, ensuring it always has a decimal point
            let s = format!("{}", f);
            if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                output.push_str(&format!("{}.0", s));
            } else {
                output.push_str(&s);
            }
        }
    }
}

/// Format a table.
fn format_table(table: &Table, output: &mut String) {
    output.push_str("{ ");
    for (i, entry) in table.entries.iter().enumerate() {
        if i > 0 {
            output.push_str(", ");
        }
        format_assignment(entry, output);
    }
    output.push_str(" }");
}

/// Format a list.
fn format_list(list: &List, output: &mut String) {
    output.push('[');
    for (i, element) in list.elements.iter().enumerate() {
        if i > 0 {
            output.push_str(", ");
        }
        format_value(element, output);
    }
    output.push(']');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_format_simple_assignment() {
        let input = r#"name = "Gum""#;
        let doc = parse(input).unwrap();
        let output = format(&doc);
        assert_eq!(output.trim(), input);
    }

    #[test]
    fn test_format_number() {
        let input = r#"count = 42"#;
        let doc = parse(input).unwrap();
        let output = format(&doc);
        assert_eq!(output.trim(), input);
    }

    #[test]
    fn test_format_boolean() {
        let input = r#"enabled = true"#;
        let doc = parse(input).unwrap();
        let output = format(&doc);
        assert_eq!(output.trim(), input);
    }

    #[test]
    fn test_format_null() {
        let input = r#"value = null"#;
        let doc = parse(input).unwrap();
        let output = format(&doc);
        assert_eq!(output.trim(), input);
    }

    #[test]
    fn test_format_table() {
        let input = r#"config = { name = "test", version = 1 }"#;
        let doc = parse(input).unwrap();
        let output = format(&doc);
        assert!(output.contains("config"));
        assert!(output.contains("name"));
        assert!(output.contains("test"));
    }

    #[test]
    fn test_format_list() {
        let input = r#"items = [1, 2, 3]"#;
        let doc = parse(input).unwrap();
        let output = format(&doc);
        assert!(output.contains("items"));
        assert!(output.contains("["));
        assert!(output.contains("]"));
    }

    #[test]
    fn test_roundtrip() {
        let input = r#"
name = "Gum"
version = 1
enabled = true
value = null
"#;
        let doc = parse(input).unwrap();
        let output = format(&doc);
        let doc2 = parse(&output).unwrap();
        assert_eq!(doc, doc2);
    }
}
