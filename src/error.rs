//! Error types.

use std::fmt;
use unicode_width::UnicodeWidthStr;

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

impl From<(&usize, &usize)> for SourcePos {
    fn from(pos: (&usize, &usize)) -> Self {
        SourcePos(pos.0.clone(), pos.1.clone())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ErrorInfo<'source> {
    source: &'source str,
    file_name: String,
    source_pos: SourcePos,
    notes: Vec<String>,
}

impl<'source> ErrorInfo<'source> {
    pub fn new(
        source: &'source str,
        file_name: &str,
        source_pos: SourcePos,
    ) -> Self {
        Self {
            source,
            file_name: file_name.to_string(),
            source_pos,
            notes: vec![],
        }
    }

    pub fn new_notes(
        source: &'source str,
        file_name: &str,
        source_pos: SourcePos,
        notes: Vec<String>,
    ) -> Self {
        let mut info = ErrorInfo::new(source, file_name, source_pos);
        info.notes = notes;
        info
    }

    pub fn source(&self) -> &'source str {
        self.source
    }
    pub fn position(&self) -> &SourcePos {
        &self.source_pos
    }
}

#[derive(Eq, PartialEq)]
pub enum Error<'source> {
    Syntax(SyntaxError<'source>),
    Render(RenderError<'source>),
    TemplateNotFound(String),
}

impl fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Syntax(ref e) => fmt::Display::fmt(e, f),
            Self::Render(ref e) => fmt::Display::fmt(e, f),
            Self::TemplateNotFound(ref name) => {
                write!(f, "Template not found {}", name) 
            },
        }
    }
}

impl fmt::Debug for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Syntax(ref e) => fmt::Debug::fmt(e, f),
            Self::Render(ref e) => fmt::Debug::fmt(e, f),
            Self::TemplateNotFound(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

impl<'source> From<RenderError<'source>> for Error<'source> {
    fn from(err: RenderError<'source>) -> Self {
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

#[derive(thiserror::Error)]
pub enum RenderError<'source> {
    #[error("Partial '{0}' not found")]
    PartialNotFound(&'source str),

    #[error(transparent)]
    Helper(#[from] HelperError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl fmt::Debug for RenderError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::PartialNotFound(_) => fmt::Display::fmt(self, f),
            Self::Helper(ref e) => fmt::Display::fmt(self, f),
            Self::Io(ref e) => fmt::Debug::fmt(e, f),
            Self::Json(ref e) => fmt::Debug::fmt(e, f),
        }
    }
}

impl PartialEq for RenderError<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::PartialNotFound(ref s), Self::PartialNotFound(ref o)) => {
                s == o
            }
            _ => false,
        }
    }
}

impl Eq for RenderError<'_> {}

#[derive(thiserror::Error, Debug)]
pub enum HelperError {
    /// Generic error message for helpers.
    #[error("{0}")]
    Message(String),

    /// Wrapper for render errors that occur via helpers.
    #[error("{0}")]
    Render(String),

    /// Error when supplied arguments do not match an exact arity.
    #[error("Helper '{0}' got invalid arity expects {1} arguments(s)")]
    ArityExact(String, usize),

    /// Error when supplied arguments do not match an arity range.
    #[error("Helper '{0}' got invalid arity expects {1}-{2} argument(s)")]
    ArityRange(String, usize, usize),

    /// Error when a helper expects a string argument.
    #[error("Helper '{0}' got invalid argument at index {1}, string expected")]
    ArgumentTypeString(String, usize),

    /// Transparent wrapper for input output errors.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Transparent wrapper for JSON errors.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

