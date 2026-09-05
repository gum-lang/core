//! Parser implementation using nom.

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, multispace0, none_of, one_of},
    combinator::{map, opt, recognize, value},
    multi::{many0, many1},
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};

use crate::ast::*;
use crate::error::{GumError, Result};

/// Reserved keywords that cannot be used as bare keys.
const RESERVED_KEYWORDS: &[&str] = &["true", "false", "null"];

/// Parse a complete Gum document.
pub fn parse(input: &str) -> Result<Document> {
    match parse_document(input) {
        Ok(("", doc)) => Ok(doc),
        Ok((remaining, _)) => {
            let (line, col) = calculate_position(input, input.len() - remaining.len());
            Err(GumError::parse(
                line,
                col,
                "Unexpected content after document",
            ))
        }
        Err(e) => Err(convert_nom_error(input, e)),
    }
}

/// Internal parser for the document.
fn parse_document(input: &str) -> IResult<&str, Document> {
    let (input, elements) = many0(alt((
        // Try to parse content (one or more assignments)
        map(parse_content, |elems| elems),
        // Try to parse non-assignment (comments, whitespace, newlines)
        map(parse_non_assignment, |na| {
            vec![DocumentElement::NonAssignment(na)]
        }),
    )))(input)?;

    // Consume any trailing whitespace
    let (input, _) = multispace0(input)?;

    let elements = elements.into_iter().flatten().collect();
    Ok((input, Document { elements }))
}

/// Parse a content block (one or more assignments with separators).
fn parse_content(input: &str) -> IResult<&str, Vec<DocumentElement>> {
    let (input, first) = parse_assignment(input)?;
    let (input, rest) = many0(preceded(parse_separator, parse_assignment))(input)?;
    let (input, _) = opt(parse_separator)(input)?;

    let mut elements = vec![DocumentElement::Assignment(first)];
    elements.extend(rest.into_iter().map(DocumentElement::Assignment));

    Ok((input, elements))
}

/// Parse an assignment: key = value
fn parse_assignment(input: &str) -> IResult<&str, Assignment> {
    let (input, _) = multispace0(input)?;
    let (input, key) = parse_key(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, val) = parse_value(input)?;

    Ok((input, Assignment { key, value: val }))
}

/// Parse non-assignment content (whitespace, comments, newlines).
fn parse_non_assignment(input: &str) -> IResult<&str, NonAssignment> {
    let (input, content) = recognize(many1(alt((
        value((), parse_comment),
        value((), parse_newline),
        value((), parse_ws),
    ))))(input)?;

    Ok((
        input,
        NonAssignment {
            content: content.to_string(),
        },
    ))
}

/// Parse a separator (comma or newline with surrounding whitespace and comments).
fn parse_separator(input: &str) -> IResult<&str, ()> {
    // *( ws / comment )
    let (input, _) = many0(alt((value((), parse_ws), value((), parse_comment))))(input)?;

    // ( comma / newline )
    let (input, _) = alt((value((), char(',')), value((), parse_newline)))(input)?;

    // *( ws / newline / comment )
    let (input, _) = many0(alt((
        value((), parse_ws),
        value((), parse_newline),
        value((), parse_comment),
    )))(input)?;

    Ok((input, ()))
}

/// Parse a key (bare, quoted, or dotted).
fn parse_key(input: &str) -> IResult<&str, Key> {
    alt((parse_dotted_key, parse_single_key))(input)
}

/// Parse a dotted key: key.key.key
fn parse_dotted_key(input: &str) -> IResult<&str, Key> {
    let (input, first) = parse_single_key(input)?;
    let (input, rest) = many1(preceded(
        tuple((multispace0, char('.'), multispace0)),
        parse_single_key,
    ))(input)?;

    if rest.is_empty() {
        Ok((input, first))
    } else {
        let mut keys = vec![first];
        keys.extend(rest);
        Ok((input, Key::Dotted(keys)))
    }
}

/// Parse a single key (bare or quoted).
fn parse_single_key(input: &str) -> IResult<&str, Key> {
    alt((parse_bare_key, parse_quoted_key))(input)
}

/// Parse a bare key (identifier).
fn parse_bare_key(input: &str) -> IResult<&str, Key> {
    let (input, key) = recognize(pair(
        alt((nom::character::complete::alpha1, tag("_"))),
        many0(alt((nom::character::complete::alphanumeric1, tag("_")))),
    ))(input)?;

    // Check for reserved keywords
    if RESERVED_KEYWORDS.contains(&key) {
        return Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    Ok((input, Key::Bare(key.to_string())))
}

/// Parse a quoted key (string).
fn parse_quoted_key(input: &str) -> IResult<&str, Key> {
    alt((
        map(parse_basic_string, Key::Quoted),
        map(parse_literal_string, Key::Quoted),
    ))(input)
}

/// Parse a value.
fn parse_value(input: &str) -> IResult<&str, Value> {
    alt((
        parse_boolean,
        parse_null,
        parse_number,
        parse_string,
        parse_table,
        parse_list,
    ))(input)
}

/// Parse a boolean value.
fn parse_boolean(input: &str) -> IResult<&str, Value> {
    alt((
        value(Value::Boolean(true), tag("true")),
        value(Value::Boolean(false), tag("false")),
    ))(input)
}

/// Parse a null value.
fn parse_null(input: &str) -> IResult<&str, Value> {
    value(Value::Null, tag("null"))(input)
}

/// Parse a number (integer or float).
fn parse_number(input: &str) -> IResult<&str, Value> {
    let (input, num_str) = recognize(tuple((
        opt(one_of("+-")),
        alt((parse_float, parse_integer)),
    )))(input)?;

    // Try to parse as integer first
    if let Ok(i) = num_str.parse::<i64>() {
        return Ok((input, Value::Number(NumberValue::Integer(i))));
    }

    // Then as float
    if let Ok(f) = num_str.parse::<f64>() {
        return Ok((input, Value::Number(NumberValue::Float(f))));
    }

    Err(nom::Err::Failure(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Float,
    )))
}

/// Parse an integer.
fn parse_integer(input: &str) -> IResult<&str, &str> {
    digit1(input)
}

/// Parse a float.
fn parse_float(input: &str) -> IResult<&str, &str> {
    recognize(alt((
        // digits.digits with optional exponent
        recognize(tuple((digit1, char('.'), digit1, opt(parse_exponent)))),
        // digits with exponent (no decimal point)
        recognize(tuple((digit1, parse_exponent))),
    )))(input)
}

/// Parse an exponent part.
fn parse_exponent(input: &str) -> IResult<&str, &str> {
    recognize(tuple((one_of("eE"), opt(one_of("+-")), digit1)))(input)
}

/// Parse a string value.
fn parse_string(input: &str) -> IResult<&str, Value> {
    alt((
        map(parse_multiline_basic_string, |s| {
            Value::String(StringValue::MultilineBasic(s))
        }),
        map(parse_multiline_literal_string, |s| {
            Value::String(StringValue::MultilineLiteral(s))
        }),
        map(parse_basic_string, |s| Value::String(StringValue::Basic(s))),
        map(parse_literal_string, |s| {
            Value::String(StringValue::Literal(s))
        }),
    ))(input)
}

/// Parse a basic string (double-quoted).
fn parse_basic_string(input: &str) -> IResult<&str, String> {
    let (input, _) = char('"')(input)?;
    let (input, content) = many0(alt((
        parse_escape_sequence,
        map(recognize(none_of("\"\\\n")), |s: &str| s.to_string()),
    )))(input)?;
    let (input, _) = char('"')(input)?;

    Ok((input, content.join("")))
}

/// Parse a multiline basic string (triple double-quoted).
fn parse_multiline_basic_string(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("\"\"\"")(input)?;
    let (input, _) = opt(char('\n'))(input)?; // Skip first newline

    // Parse content until we see """
    let mut content = String::new();
    let mut remaining = input;

    loop {
        // Check if we've reached the closing """
        if remaining.starts_with("\"\"\"") {
            break;
        }

        // Try to parse an escape sequence
        if let Ok((rest, escaped)) = parse_escape_sequence(remaining) {
            content.push_str(&escaped);
            remaining = rest;
            continue;
        }

        // Try to parse a line continuation
        if let Ok((rest, _)) = parse_line_continuation(remaining) {
            remaining = rest;
            continue;
        }

        // Parse a single character
        if remaining.is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(
                remaining,
                nom::error::ErrorKind::Eof,
            )));
        }

        let ch = remaining.chars().next().unwrap();
        content.push(ch);
        remaining = &remaining[ch.len_utf8()..];
    }

    let (remaining, _) = tag("\"\"\"")(remaining)?;

    // Remove common leading whitespace (dedent)
    content = dedent(&content);

    Ok((remaining, content))
}

/// Parse a literal string (single-quoted).
fn parse_literal_string(input: &str) -> IResult<&str, String> {
    let (input, _) = char('\'')(input)?;
    let (input, content) = recognize(many0(none_of("'")))(input)?;
    let (input, _) = char('\'')(input)?;

    Ok((input, content.to_string()))
}

/// Parse a multiline literal string (triple single-quoted).
fn parse_multiline_literal_string(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("'''")(input)?;
    let (input, _) = opt(char('\n'))(input)?; // Skip first newline

    // Parse content until we see '''
    let mut content = String::new();
    let mut remaining = input;

    loop {
        // Check if we've reached the closing '''
        if remaining.starts_with("'''") {
            break;
        }

        // Try to parse a line continuation
        if let Ok((rest, _)) = parse_line_continuation(remaining) {
            remaining = rest;
            continue;
        }

        // Parse a single character
        if remaining.is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(
                remaining,
                nom::error::ErrorKind::Eof,
            )));
        }

        let ch = remaining.chars().next().unwrap();
        content.push(ch);
        remaining = &remaining[ch.len_utf8()..];
    }

    let (remaining, _) = tag("'''")(remaining)?;

    // Remove common leading whitespace (dedent)
    content = dedent(&content);

    Ok((remaining, content))
}

/// Parse an escape sequence.
fn parse_escape_sequence(input: &str) -> IResult<&str, String> {
    let (input, _) = char('\\')(input)?;
    let (input, escaped) = alt((
        value('"', char('"')),
        value('\\', char('\\')),
        value('\x08', char('b')),
        value('\x0C', char('f')),
        value('\n', char('n')),
        value('\r', char('r')),
        value('\t', char('t')),
        parse_unicode_escape,
    ))(input)?;

    Ok((input, escaped.to_string()))
}

/// Parse a unicode escape sequence.
fn parse_unicode_escape(input: &str) -> IResult<&str, char> {
    let (input, _) = one_of("uU")(input)?;
    let (input, hex) = alt((recognize(many1(nom::character::complete::one_of(
        "0123456789ABCDEFabcdef",
    ))),))(input)?;

    let code_point = u32::from_str_radix(hex, 16).map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::HexDigit,
        ))
    })?;

    let ch = char::from_u32(code_point).ok_or_else(|| {
        nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Char))
    })?;

    Ok((input, ch))
}

/// Parse a line continuation (backslash followed by newline and whitespace).
fn parse_line_continuation(input: &str) -> IResult<&str, String> {
    let (input, _) = char('\\')(input)?;
    let (input, _) = parse_newline(input)?;
    let (input, _ws) = parse_ws(input)?;

    Ok((input, String::new())) // Line continuation produces empty string
}

/// Parse a table (inline key-value pairs).
fn parse_table(input: &str) -> IResult<&str, Value> {
    let (input, _) = char('{')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, entries) = many0(terminated(parse_assignment, opt(parse_separator)))(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('}')(input)?;

    Ok((input, Value::Table(Table { entries })))
}

/// Parse a list (ordered sequence of values).
fn parse_list(input: &str) -> IResult<&str, Value> {
    let (input, _) = char('[')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, elements) = many0(terminated(parse_value, opt(parse_separator)))(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char(']')(input)?;

    Ok((input, Value::List(List { elements })))
}

/// Parse whitespace (at least one space or tab).
fn parse_ws(input: &str) -> IResult<&str, &str> {
    take_while1(|c| c == ' ' || c == '\t')(input)
}

/// Parse a newline.
fn parse_newline(input: &str) -> IResult<&str, &str> {
    alt((tag("\r\n"), tag("\n")))(input)
}

/// Parse a comment.
fn parse_comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = char('#')(input)?;
    let (input, content) = take_while(|c| c != '\n' && c != '\r')(input)?;
    Ok((input, content))
}

/// Remove common leading whitespace from multiline strings (Python dedent algorithm).
fn dedent(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return text.to_string();
    }

    // Find minimum indentation (excluding empty lines)
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    // Remove the common indentation
    lines
        .iter()
        .map(|line| {
            if line.len() >= min_indent {
                &line[min_indent..]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Calculate line and column from byte offset.
fn calculate_position(input: &str, offset: usize) -> (usize, usize) {
    let prefix = &input[..offset];
    let line = prefix.matches('\n').count() + 1;
    let last_newline = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let column = offset - last_newline + 1;
    (line, column)
}

/// Convert a nom error to a GumError.
fn convert_nom_error(input: &str, err: nom::Err<nom::error::Error<&str>>) -> GumError {
    match err {
        nom::Err::Incomplete(_) => GumError::UnexpectedEof,
        nom::Err::Error(e) | nom::Err::Failure(e) => {
            let offset = input.len() - e.input.len();
            let (line, col) = calculate_position(input, offset);
            GumError::parse(line, col, format!("Parse error: {:?}", e.code))
        }
    }
}
