//! Utilties for generating error messages with source code.
use std::fmt;
use unicode_width::UnicodeWidthStr;

use crate::parser::ParseState;

/// Map a position for syntax errors.
#[derive(Debug, Eq, PartialEq)]
pub struct SourcePos(pub usize, pub usize);

impl SourcePos {
    /// The line number for this source position.
    pub fn line(&self) -> &usize {
        &self.0
    }

    /// The byte offset for this source position.
    pub fn byte_offset(&self) -> &usize {
        &self.1
    }
}

impl From<(&usize, &usize)> for SourcePos {
    fn from(pos: (&usize, &usize)) -> Self {
        SourcePos(pos.0.clone(), pos.1.clone())
    }
}

/// Information needed to generate a source code snippet.
#[derive(Eq, PartialEq)]
pub struct ErrorInfo<'source> {
    source: &'source str,
    file_name: String,
    source_pos: SourcePos,
    notes: Vec<String>,
}

impl<'source> ErrorInfo<'source> {
    /// Create a new error info.
    pub fn new(
        source: &'source str,
        file_name: &str,
        source_pos: SourcePos,
        notes: Vec<String>,
    ) -> Self {
        Self {
            source,
            file_name: file_name.to_string(),
            source_pos,
            notes,
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

impl<'source> From<(&'source str, &mut ParseState)> for ErrorInfo<'source> {
    fn from(source: (&'source str, &mut ParseState)) -> Self {
        ErrorInfo::new(
            source.0,
            source.1.file_name(),
            SourcePos::from((
                source.1.line(),
                source.1.byte(),
            )),
            vec![],
        )
    }
}

impl<'source> From<(&'source str, &mut ParseState, Vec<String>)> for ErrorInfo<'source> {
    fn from(source: (&'source str, &mut ParseState, Vec<String>)) -> Self {
        ErrorInfo::new(
            source.0,
            source.1.file_name(),
            SourcePos::from((
                source.1.line(),
                source.1.byte(),
            )),
            source.2,
        )
    }
}

impl fmt::Debug for ErrorInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.source;
        let pos = &self.source_pos;
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
            format!("{}:{}:{}", self.file_name, line_number + 1, cols);

        let err_pointer: String = if cols > 0 {
            format!("{}^", "-".repeat(cols - 1))
        } else {
            "^".to_string()
        };

        //write!(f, "error: {}\n", self.to_string())?;
        write!(f, "{}--> {}\n", line_padding, file_info)?;
        write!(f, "{} |\n", line_padding)?;
        write!(f, "{}{}\n", line_prefix, line_slice)?;
        write!(f, "{} | {}", line_padding, err_pointer)?;

        if !self.notes.is_empty() {
            write!(f, "\n")?;
            for n in self.notes.iter() {
                write!(f, "{} = note: {}", line_padding, n)?;
            }
        }

        Ok(())
    }
}

impl Into<String> for ErrorInfo<'_> {
    fn into(self) -> String {
        format!("{:?}", self)
    }
}
