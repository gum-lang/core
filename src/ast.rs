//! Abstract Syntax Tree types for Gum documents.

use serde::{Deserialize, Serialize};

/// A complete Gum document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Top-level content (assignments and non-assignment elements).
    pub elements: Vec<DocumentElement>,
}

/// An element at the document level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentElement {
    /// A key-value assignment.
    Assignment(Assignment),
    /// Non-assignment content (whitespace, comments, newlines).
    NonAssignment(NonAssignment),
}

/// A key-value assignment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    /// The key being assigned.
    pub key: Key,
    /// The value being assigned.
    pub value: Value,
}

/// Non-assignment content (whitespace, comments, newlines).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NonAssignment {
    /// The raw content.
    pub content: String,
}

/// A key in an assignment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Key {
    /// A bare key (unquoted identifier).
    Bare(String),
    /// A quoted key (single or double quoted).
    Quoted(String),
    /// A dotted key (multiple keys separated by dots).
    Dotted(Vec<Key>),
}

/// A value in an assignment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// A string value.
    String(StringValue),
    /// A numeric value.
    Number(NumberValue),
    /// A boolean value.
    Boolean(bool),
    /// A null value.
    Null,
    /// A table (inline key-value pairs).
    Table(Table),
    /// A list (ordered sequence of values).
    List(List),
}

/// A string value with its original form.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StringValue {
    /// A basic quoted string: "text"
    Basic(String),
    /// A multiline quoted string: """text"""
    MultilineBasic(String),
    /// A literal string: 'text'
    Literal(String),
    /// A multiline literal string: '''text'''
    MultilineLiteral(String),
}

/// A numeric value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NumberValue {
    /// An integer value.
    Integer(i64),
    /// A floating-point value.
    Float(f64),
}

/// A table (inline key-value pairs).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Table {
    /// The table's entries.
    pub entries: Vec<Assignment>,
}

/// A list (ordered sequence of values).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct List {
    /// The list's elements.
    pub elements: Vec<Value>,
}

impl Document {
    /// Create a new empty document.
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Get all assignments in the document.
    pub fn assignments(&self) -> Vec<&Assignment> {
        self.elements
            .iter()
            .filter_map(|e| match e {
                DocumentElement::Assignment(a) => Some(a),
                _ => None,
            })
            .collect()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Key {
    /// Get the string representation of the key.
    pub fn as_str(&self) -> String {
        match self {
            Key::Bare(s) => s.clone(),
            Key::Quoted(s) => s.clone(),
            Key::Dotted(keys) => keys
                .iter()
                .map(|k| k.as_str())
                .collect::<Vec<_>>()
                .join("."),
        }
    }
}

impl Value {
    /// Check if this value is a string.
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Check if this value is a number.
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// Check if this value is a boolean.
    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    /// Check if this value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Check if this value is a table.
    pub fn is_table(&self) -> bool {
        matches!(self, Value::Table(_))
    }

    /// Check if this value is a list.
    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }
}
