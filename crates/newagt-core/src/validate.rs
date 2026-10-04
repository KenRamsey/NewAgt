//! Structured validation report (M6): parse warnings + post-parse AST checks.

use crate::ast::{Agt, AgtItem, Document, PrjItem, PrjSect};
use crate::extension::{ParseWarning, ParseWarningKind};
use crate::keyword::Keyword;
use crate::parse::{parse_with_options, Found, ParseError, ParseOptions};
use crate::span::Span;

/// Severity for validation findings (reporting, not gatekeeping).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

impl ValidationSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

/// Stable category code for tooling and JSON export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationCode {
    UnbalancedBraces,
    UnterminatedString,
    LexError,
    ParseError,
    ValueError,
    ExtraInput,
    UnknownKeyword,
    OddPlacement,
    ProfileExtension,
    DuplicateSingleton,
    MissingOptionalSection,
}

impl ValidationCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnbalancedBraces => "unbalanced_braces",
            Self::UnterminatedString => "unterminated_string",
            Self::LexError => "lex_error",
            Self::ParseError => "parse_error",
            Self::ValueError => "value_error",
            Self::ExtraInput => "extra_input",
            Self::UnknownKeyword => "unknown_keyword",
            Self::OddPlacement => "odd_placement",
            Self::ProfileExtension => "profile_extension",
            Self::DuplicateSingleton => "duplicate_singleton",
            Self::MissingOptionalSection => "missing_optional_section",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationEntry {
    pub severity: ValidationSeverity,
    pub message: String,
    pub span: Span,
    pub code: ValidationCode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    /// Document parsed successfully (Ken corpus loadability).
    pub loadable: bool,
    pub entries: Vec<ValidationEntry>,
}

impl ValidationReport {
    pub fn has_errors(&self) -> bool {
        self.entries
            .iter()
            .any(|e| e.severity == ValidationSeverity::Error)
    }
}

/// Validate `source` under `options`: aggregate parse warnings and AST checks.
pub fn validate(source: &str, options: ParseOptions) -> ValidationReport {
    let mut entries = Vec::new();
    if let Some(span) = find_unterminated_string_at_eof(source) {
        entries.push(ValidationEntry {
            severity: ValidationSeverity::Error,
            message: "string literal not closed before end of file".to_string(),
            span,
            code: ValidationCode::UnterminatedString,
        });
    }
    if let Some(span) = find_unbalanced_braces(source) {
        entries.push(ValidationEntry {
            severity: ValidationSeverity::Error,
            message: "unbalanced `{` and `}` in source".to_string(),
            span,
            code: ValidationCode::UnbalancedBraces,
        });
    }

    match parse_with_options(source, options) {
        Ok(result) => {
            for warning in result.warnings {
                entries.push(warning_to_entry(warning));
            }
            check_document(&result.document, &mut entries);
            ValidationReport {
                loadable: true,
                entries,
            }
        }
        Err(err) => {
            entries.push(parse_error_to_entry(&err));
            ValidationReport {
                loadable: false,
                entries,
            }
        }
    }
}

fn warning_to_entry(w: ParseWarning) -> ValidationEntry {
    let (code, severity) = match w.kind {
        ParseWarningKind::UnknownKeyword => (ValidationCode::UnknownKeyword, ValidationSeverity::Warning),
        ParseWarningKind::OddPlacement => (ValidationCode::OddPlacement, ValidationSeverity::Warning),
        ParseWarningKind::ProfileExtension => {
            (ValidationCode::ProfileExtension, ValidationSeverity::Warning)
        }
    };
    ValidationEntry {
        severity,
        message: w.message,
        span: w.span,
        code,
    }
}

fn parse_error_to_entry(err: &ParseError) -> ValidationEntry {
    match err {
        ParseError::Lex(e) => ValidationEntry {
            severity: ValidationSeverity::Error,
            message: e.to_string(),
            span: lex_error_span(e),
            code: ValidationCode::LexError,
        },
        ParseError::Value(e) => ValidationEntry {
            severity: ValidationSeverity::Error,
            message: e.to_string(),
            span: e.span,
            code: ValidationCode::ValueError,
        },
        ParseError::Unexpected {
            span,
            expected,
            found,
        } => {
            let code = if matches!(found, Found::End) && expected.contains('}') {
                ValidationCode::UnbalancedBraces
            } else {
                ValidationCode::ParseError
            };
            ValidationEntry {
                severity: ValidationSeverity::Error,
                message: err.to_string(),
                span: *span,
                code,
            }
        }
        ParseError::ExtraInput { span } => ValidationEntry {
            severity: ValidationSeverity::Error,
            message: err.to_string(),
            span: *span,
            code: ValidationCode::ExtraInput,
        },
    }
}

fn lex_error_span(err: &crate::lex::LexError) -> Span {
    match err {
        crate::lex::LexError::UnexpectedCharacter { span, .. }
        | crate::lex::LexError::IntegerOverflow { span }
        | crate::lex::LexError::InvalidReal { span } => *span,
    }
}

/// Scan for a `"` opened but not closed before EOF (PDF-style; newline still closes in lex).
fn find_unterminated_string_at_eof(source: &str) -> Option<Span> {
    let bytes = source.as_bytes();
    let mut line = 1u32;
    let mut column = 1u32;
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'"' {
            let start_line = line;
            let start_col = column;
            let start_offset = i;
            i += 1;
            column += 1;
            let mut closed = false;
            while i < bytes.len() {
                let c = bytes[i];
                if c == b'"' {
                    closed = true;
                    i += 1;
                    column += 1;
                    break;
                }
                if c == b'\n' {
                    // Lex treats newline as end of string — not EOF-unterminated.
                    i += 1;
                    line += 1;
                    column = 1;
                    closed = true;
                    break;
                }
                i += 1;
                column += 1;
            }
            if !closed {
                return Some(Span::new(start_line, start_col, start_offset));
            }
            continue;
        }
        if b == b'\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
        i += 1;
    }
    None
}

/// Brace balance outside of string literals.
fn find_unbalanced_braces(source: &str) -> Option<Span> {
    let bytes = source.as_bytes();
    let mut depth = 0i32;
    let mut line = 1u32;
    let mut column = 1u32;
    let mut i = 0usize;
    let mut first_open: Option<(u32, u32, usize)> = None;

    while i < bytes.len() {
        let b = bytes[i];
        if b == b'"' {
            i += 1;
            column += 1;
            while i < bytes.len() {
                let c = bytes[i];
                if c == b'"' {
                    i += 1;
                    column += 1;
                    break;
                }
                if c == b'\n' {
                    i += 1;
                    line += 1;
                    column = 1;
                    break;
                }
                i += 1;
                column += 1;
            }
            continue;
        }
        if b == b'{' {
            if depth == 0 {
                first_open = Some((line, column, i));
            }
            depth += 1;
        } else if b == b'}' {
            depth -= 1;
            if depth < 0 {
                return Some(Span::new(line, column, i));
            }
        }
        if b == b'\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
        i += 1;
    }
    if depth != 0 {
        let (l, c, o) = first_open.unwrap_or((1, 1, 0));
        Some(Span::new(l, c, o))
    } else {
        None
    }
}

fn check_document(doc: &Document, entries: &mut Vec<ValidationEntry>) {
    check_agt(&doc.root, entries);
}

fn check_agt(agt: &Agt, entries: &mut Vec<ValidationEntry>) {
    let mut prj = 0u32;
    let mut sen = 0u32;
    let mut tgt = 0u32;

    for item in &agt.items {
        match item {
            AgtItem::PrjSect(sect) => {
                prj += 1;
                if prj > 1 {
                    push_duplicate(entries, "PrjSect", sect.span);
                }
                check_prj_sect(sect, entries);
            }
            AgtItem::SenSect(_) => {
                sen += 1;
                if sen > 1 {
                    let span = match item {
                        AgtItem::SenSect(s) => s.span,
                        _ => unreachable!(),
                    };
                    push_duplicate(entries, "SenSect", span);
                }
            }
            AgtItem::TgtSect(_) => {
                tgt += 1;
                if tgt > 1 {
                    let span = match item {
                        AgtItem::TgtSect(s) => s.span,
                        _ => unreachable!(),
                    };
                    push_duplicate(entries, "TgtSect", span);
                }
            }
            AgtItem::Unknown(_) => {}
        }
    }

    if prj == 0 {
        push_missing_section(entries, "PrjSect", agt.span);
    }
    if sen == 0 {
        push_missing_section(entries, "SenSect", agt.span);
    }
    if tgt == 0 {
        push_missing_section(entries, "TgtSect", agt.span);
    }
}

fn check_prj_sect(sect: &PrjSect, entries: &mut Vec<ValidationEntry>) {
    let singletons = [
        Keyword::Name,
        Keyword::Scenario,
        Keyword::Site,
        Keyword::Time,
        Keyword::LatLong,
        Keyword::Utm,
    ];
    let mut counts = [0u32; 6];
    for item in &sect.items {
        if let PrjItem::Field(field) = item {
            if let Some(idx) = singletons.iter().position(|k| *k == field.keyword) {
                counts[idx] += 1;
                if counts[idx] > 1 {
                    entries.push(ValidationEntry {
                        severity: ValidationSeverity::Warning,
                        message: format!(
                            "duplicate `{}` in PrjSect (spec implies at most one)",
                            field.keyword
                        ),
                        span: field.keyword_span,
                        code: ValidationCode::DuplicateSingleton,
                    });
                }
            }
        }
    }
}

fn push_duplicate(entries: &mut Vec<ValidationEntry>, name: &str, span: Span) {
    entries.push(ValidationEntry {
        severity: ValidationSeverity::Warning,
        message: format!("duplicate `{name}` section in Agt (spec implies at most one)"),
        span,
        code: ValidationCode::DuplicateSingleton,
    });
}

fn push_missing_section(entries: &mut Vec<ValidationEntry>, name: &str, span: Span) {
    entries.push(ValidationEntry {
        severity: ValidationSeverity::Info,
        message: format!("no `{name}` in Agt container"),
        span,
        code: ValidationCode::MissingOptionalSection,
    });
}

/// Human-readable report for CLI stdout.
pub fn format_report_text(report: &ValidationReport) -> String {
    let mut out = String::new();
    out.push_str(if report.loadable {
        "validation: loadable\n"
    } else {
        "validation: not loadable\n"
    });
    for entry in &report.entries {
        out.push_str(&format!(
            "{} [{}] line {}, column {}: {}\n",
            entry.severity.as_str(),
            entry.code.as_str(),
            entry.span.line,
            entry.span.column,
            entry.message
        ));
    }
    out
}

/// JSON report (`newagt validate --format json`).
pub fn format_report_json(report: &ValidationReport) -> String {
    let mut out = String::from("{\"loadable\":");
    out.push_str(if report.loadable { "true" } else { "false" });
    out.push_str(",\"entries\":[");
    for (i, entry) in report.entries.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"severity\":");
        out.push_str(&json_string(entry.severity.as_str()));
        out.push_str(",\"code\":");
        out.push_str(&json_string(entry.code.as_str()));
        out.push_str(",\"message\":");
        out.push_str(&json_string(&entry.message));
        out.push_str(",\"span\":{\"line\":");
        out.push_str(&entry.span.line.to_string());
        out.push_str(",\"column\":");
        out.push_str(&entry.span.column.to_string());
        out.push_str(",\"offset\":");
        out.push_str(&entry.span.offset.to_string());
        out.push_str("}}");
    }
    out.push_str("]}");
    out
}

fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_with_warnings;

    #[test]
    fn valid_minimal_loadable_with_missing_section_info() {
        let src = r#"Agt { TgtSect { TgtUpd { Tgt { Name "t1" } } } }"#;
        let report = validate(src, ParseOptions::default());
        assert!(report.loadable);
        assert!(report
            .entries
            .iter()
            .any(|e| e.code == ValidationCode::MissingOptionalSection
                && e.message.contains("PrjSect")));
    }

    #[test]
    fn broken_brace_not_loadable() {
        let src = r#"Agt { PrjSect { Name "x""#;
        let report = validate(src, ParseOptions::default());
        assert!(!report.loadable);
        assert!(report.entries.iter().any(|e| {
            e.severity == ValidationSeverity::Error
                && (e.code == ValidationCode::UnbalancedBraces
                    || e.code == ValidationCode::ParseError)
        }));
    }

    #[test]
    fn parse_warnings_surface_as_validation_warnings() {
        let src = r#"Agt { PrjSect { CustomTag "x" } }"#;
        let report = validate(src, ParseOptions::default());
        assert!(report.loadable);
        assert!(report.entries.iter().any(|e| {
            e.code == ValidationCode::UnknownKeyword && e.severity == ValidationSeverity::Warning
        }));
    }

    #[test]
    fn json_format_contains_loadable_and_severity() {
        let result = parse_with_warnings(r#"Agt { PrjSect { Name "p" } }"#).unwrap();
        let report = ValidationReport {
            loadable: true,
            entries: result
                .warnings
                .into_iter()
                .map(warning_to_entry)
                .collect(),
        };
        let json = format_report_json(&report);
        assert!(json.contains("\"loadable\":true"));
        assert!(json.contains("\"entries\":["));
    }

    #[test]
    fn unterminated_string_at_eof_is_error_entry() {
        let src = r#"Agt { PrjSect { Name "no closing quote } }"#;
        let report = validate(src, ParseOptions::default());
        assert!(report.entries.iter().any(|e| {
            e.code == ValidationCode::UnterminatedString
                && e.severity == ValidationSeverity::Error
        }));
    }
}
