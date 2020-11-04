//! Errors generated when compiling templates.
use std::fmt;
use unicode_width::UnicodeWidthStr;
use crate::error::{ErrorInfo, SourcePos};

static SYNTAX_PREFIX: &str = "Syntax error";

#[derive(Eq, PartialEq)]
pub enum SyntaxError<'source> {
    EmptyStatement(ErrorInfo<'source>),
    ExpectedIdentifier(ErrorInfo<'source>),
    ExpectedSimpleIdentifier(ErrorInfo<'source>),
    PartialIdentifier(ErrorInfo<'source>),
    PartialSimpleIdentifier(ErrorInfo<'source>),
    BlockIdentifier(ErrorInfo<'source>),
    OpenStatement(ErrorInfo<'source>),
    StringLiteralNewline(ErrorInfo<'source>),
    UnexpectedPathExplicitThis(ErrorInfo<'source>),
    UnexpectedPathParent(ErrorInfo<'source>),
    UnexpectedPathLocal(ErrorInfo<'source>),
    UnexpectedPathDelimiter(ErrorInfo<'source>),
    UnexpectedPathParentWithLocal(ErrorInfo<'source>),
    UnexpectedPathParentWithExplicit(ErrorInfo<'source>),
    ExpectedPathDelimiter(ErrorInfo<'source>),
    OpenSubExpression(ErrorInfo<'source>),
    TagNameMismatch(ErrorInfo<'source>),
    BlockNotOpen(ErrorInfo<'source>),
}

impl SyntaxError<'_> {
    fn message(&self) -> &'static str {
        match *self {
            Self::EmptyStatement(_) => "statement is empty",
            Self::ExpectedIdentifier(_) => "expecting identifier",
            Self::ExpectedSimpleIdentifier(_) => {
                "expecting identifier not a path or sub-expression"
            }
            Self::PartialIdentifier(_) => "partial requires an identifier",
            Self::PartialSimpleIdentifier(_) => {
                "partial requires a simple identifier (not a path)"
            }
            Self::BlockIdentifier(_) => "block scope requires an identifier",
            Self::OpenStatement(_) => "statement not terminated",
            Self::StringLiteralNewline(_) => {
                "new lines in string literals must be escaped (\\n)"
            }
            Self::UnexpectedPathExplicitThis(_) => {
                "explicit this reference must be at the start of a path"
            }
            Self::UnexpectedPathParent(_) => {
                "parent scopes must be at the start of a path"
            }
            Self::UnexpectedPathLocal(_) => {
                "local scope identifiers must be at the start of a path"
            }
            Self::UnexpectedPathDelimiter(_) => {
                "expected identifier but got path delimiter"
            }
            Self::UnexpectedPathParentWithLocal(_) => {
                "parent scopes and local identifiers are mutually exclusive"
            }
            Self::UnexpectedPathParentWithExplicit(_) => {
                "parent scopes and explicit this are mutually exclusive"
            }
            Self::ExpectedPathDelimiter(_) => "expected path delimiter (.)",
            Self::OpenSubExpression(_) => "sub-expression not terminated",
            Self::TagNameMismatch(_) => "closing name does not match",
            Self::BlockNotOpen(_) => "got a closing tag but no block is open",
        }
    }

    pub fn info(&self) -> &ErrorInfo {
        match *self {
            Self::EmptyStatement(ref info) => info,
            Self::ExpectedIdentifier(ref info) => info,
            Self::ExpectedSimpleIdentifier(ref info) => info,
            Self::PartialIdentifier(ref info) => info,
            Self::PartialSimpleIdentifier(ref info) => info,
            Self::BlockIdentifier(ref info) => info,
            Self::OpenStatement(ref info) => info,
            Self::StringLiteralNewline(ref info) => info,
            Self::UnexpectedPathExplicitThis(ref info) => info,
            Self::UnexpectedPathParent(ref info) => info,
            Self::UnexpectedPathLocal(ref info) => info,
            Self::UnexpectedPathDelimiter(ref info) => info,
            Self::UnexpectedPathParentWithLocal(ref info) => info,
            Self::UnexpectedPathParentWithExplicit(ref info) => info,
            Self::ExpectedPathDelimiter(ref info) => info,
            Self::OpenSubExpression(ref info) => info,
            Self::TagNameMismatch(ref info) => info,
            Self::BlockNotOpen(ref info) => info,
        }
    }

    fn find_prev_line_offset(&self, s: &str, pos: &SourcePos) -> Option<usize> {
        let mut counter: usize = pos.byte_offset().clone();
        while counter > 0 {
            // TODO: clamp end range to string length!
            let slice = &s[counter..counter + 1];
            if slice == "\n" {
                return Some(counter);
            }
            counter -= 1;
        }
        None
    }

    fn find_next_line_offset(&self, s: &str, pos: &SourcePos) -> Option<usize> {
        let mut counter: usize = pos.byte_offset().clone();
        while counter < s.len() {
            // TODO: clamp end range to string length!
            let slice = &s[counter..counter + 1];
            if slice == "\n" {
                return Some(counter);
            }
            counter += 1;
        }
        None
    }
}

impl fmt::Display for SyntaxError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", SYNTAX_PREFIX, self.message())
    }
}

impl fmt::Debug for SyntaxError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let info = self.info();
        let s = info.source();
        let pos = info.position();
        let prev_line = self.find_prev_line_offset(s, pos);
        let prev_line_offset = if let Some(offset) = prev_line {
            offset + 1
        } else {
            0
        };

        let next_line = self.find_next_line_offset(s, pos);
        let next_line_offset = if let Some(offset) = next_line {
            offset
        } else {
            s.len()
        };

        let line_slice = &s[prev_line_offset..next_line_offset];
        let line_number = pos.line();

        let line_prefix = format!(" {} | ", line_number + 1);
        let line_padding = " ".repeat(line_prefix.len() - 3);

        let diff = (pos.byte_offset() - prev_line_offset) + 1;
        let diff_start = prev_line_offset;
        let diff_end = prev_line_offset + diff;
        let diff_str = &s[diff_start..diff_end];

        let cols = UnicodeWidthStr::width(diff_str);

        let file_info =
            format!("{}:{}:{}", info.file_name, line_number + 1, cols);

        let err_pointer: String = if cols > 0 {
            format!("{}^", "-".repeat(cols - 1))
        } else {
            "^".to_string()
        };

        write!(f, "error: {}\n", self.to_string())?;
        write!(f, "{}--> {}\n", line_padding, file_info)?;
        write!(f, "{} |\n", line_padding)?;
        write!(f, "{}{}\n", line_prefix, line_slice)?;
        write!(f, "{} | {}", line_padding, err_pointer)?;

        if !info.notes.is_empty() {
            write!(f, "\n")?;
            for n in info.notes.iter() {
                write!(f, "{} = note: {}", line_padding, n)?;
            }
        }

        Ok(())
    }
}

