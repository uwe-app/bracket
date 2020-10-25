use std::io;
use thiserror::Error;
use unicode_width::UnicodeWidthStr;
use crate::lexer::SourcePos;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum Error {
    #[error(transparent)]
    Syntax(#[from] SyntaxError),
    #[error(transparent)]
    Render(#[from] RenderError),
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum SyntaxError {
    #[error("statement is empty")]
    EmptyStatement(SourcePos),
    #[error("expecting identifier")]
    ExpectedIdentifier(SourcePos),
}

impl SyntaxError {

    pub fn position(&self) -> &SourcePos {
        match *self {
            Self::EmptyStatement(ref pos) => pos,
            Self::ExpectedIdentifier(ref pos) => pos,
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

    fn error_context(&self, s: &str, mut w: impl io::Write) -> io::Result<()> {
        let pos = self.position();
        let prev_line = self.find_prev_line_offset(s, pos);
        let prev_line_offset = if let Some(offset) = prev_line {
            offset + 1
        } else { 0 };

        let next_line = self.find_next_line_offset(s, pos);
        let next_line_offset = if let Some(offset) = next_line {
            offset - 1
        } else { s.len() - 1 };

        let line_slice = &s[prev_line_offset..next_line_offset];
        let line_number = pos.line();

        let line_prefix = format!(" {} | ", line_number + 1);
        let line_padding = " ".repeat(line_prefix.len() - 3);

        let diff = (pos.byte_offset() - prev_line_offset) + 1;
        let diff_start = prev_line_offset;
        let diff_end = prev_line_offset + diff;
        let diff_str = &s[diff_start..diff_end];

        let cols = UnicodeWidthStr::width(diff_str);

        let file_info = format!("unknown:{}:{}", line_number + 1, cols);

        let err_pointer: String = if cols > 0 {
            format!("{}^", "-".repeat(cols - 1))
        } else { "^".to_string() };

        let mut msg = String::new();
        let err = self.to_string();
        write!(w, "error: Syntax error, {}\n", err)?;
        write!(w, "{}--> {}\n", line_padding, file_info)?;
        write!(w, "{} |\n", line_padding)?;
        write!(w, "{}{}\n", line_prefix, line_slice)?;
        write!(w, "{} | {}\n", line_padding, err_pointer)?;
        Ok(())
    }

    pub fn print(&self, s: &str) -> io::Result<()> {
        self.error_context(s, io::stderr())
    }
}

#[derive(Error, Debug)]
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
            // FIXME:
            _ => false,
        }
    }
}

impl Eq for RenderError {}
