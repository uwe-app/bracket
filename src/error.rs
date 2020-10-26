use std::fmt;
use unicode_width::UnicodeWidthStr;

use crate::lexer::parser::ParserOptions;

static SYNTAX_PREFIX: &str = "Syntax error";

/// Map a position for syntax errors.
#[derive(Debug, Eq, PartialEq)]
pub struct SourcePos(pub usize, pub usize);

impl SourcePos {
    pub fn line(&self) -> &usize {
        &self.0
    }

    pub fn byte_offset(&self) -> &usize {
        &self.1
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ErrorInfo<'source> {
    source: &'source str,
    file_name: String,
    source_pos: SourcePos,
}

impl<'source> ErrorInfo<'source> {
    pub fn source(&self) -> &'source str {
        self.source
    }
    pub fn position(&self) -> &SourcePos {
        &self.source_pos
    }
}

impl<'source> From<(&'source str, &ParserOptions, SourcePos)>
    for ErrorInfo<'source>
{
    fn from(opts: (&'source str, &ParserOptions, SourcePos)) -> Self {
        Self {
            source: opts.0,
            file_name: opts.1.file_name.clone(),
            source_pos: opts.2,
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum Error<'source> {
    Syntax(SyntaxError<'source>),
    Render(RenderError),
}

impl fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Syntax(ref e) => fmt::Display::fmt(e, f),
            Self::Render(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

impl fmt::Debug for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Syntax(ref e) => fmt::Debug::fmt(e, f),
            Self::Render(ref e) => fmt::Debug::fmt(e, f),
        }
    }
}

impl From<RenderError> for Error<'_> {
    fn from(err: RenderError) -> Self {
        Self::Render(err)
    }
}

impl<'source> From<SyntaxError<'source>> for Error<'source> {
    fn from(err: SyntaxError<'source>) -> Self {
        Self::Syntax(err)
    }
}

#[derive(Eq, PartialEq)]
pub enum SyntaxError<'source> {
    EmptyStatement(ErrorInfo<'source>),
    ExpectedIdentifier(ErrorInfo<'source>),
    PartialIdentifier(ErrorInfo<'source>),
    PartialSimpleIdentifier(ErrorInfo<'source>),
    BlockIdentifier(ErrorInfo<'source>),
}

impl SyntaxError<'_> {
    fn message(&self) -> &'static str {
        match *self {
            Self::EmptyStatement(_) => "statement is empty",
            Self::ExpectedIdentifier(_) => "expecting identifier",
            Self::PartialIdentifier(_) => "partial requires an identifier",
            Self::PartialSimpleIdentifier(_) => "partial requires a simple identifier (not a path)",
            Self::BlockIdentifier(_) => "block scope requires an identifier",
        }
    }

    pub fn info(&self) -> &ErrorInfo {
        match *self {
            Self::EmptyStatement(ref info) => info,
            Self::ExpectedIdentifier(ref info) => info,
            Self::PartialIdentifier(ref info) => info,
            Self::PartialSimpleIdentifier(ref info) => info,
            Self::BlockIdentifier(ref info) => info,
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
        write!(f, "{} | {}", line_padding, err_pointer)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RenderError {
    #[error("Template not found {0}")]
    TemplateNotFound(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl PartialEq for RenderError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::TemplateNotFound(ref s), Self::TemplateNotFound(ref o)) => {
                s == o
            }
            _ => false,
        }
    }
}

impl Eq for RenderError {}
