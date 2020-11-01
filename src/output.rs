//! Trait and type for rendering to destinations.
use std::io::{Result, Write};

/// Trait for types that we can render to.
pub trait Output: Write {
    /// Convenience function as we are typically writing string slices.
    fn write_str(&mut self, s: &str) -> Result<usize>;
}

/// Output type that wraps an `io::Write` writer.
pub struct Writer<W: Write> {
    writer: W,
}

impl<W: Write> Output for Writer<W> {
    fn write_str(&mut self, s: &str) -> Result<usize> {
        self.writer.write(s.as_bytes())
    }
}

impl<W: Write> Write for Writer<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.writer.flush()
    }
}

/// Output type that buffers into a string.
///
/// Call `into()` to access the result after
/// rendering.
pub struct StringOutput {
    value: String,
}

impl StringOutput {
    pub fn new() -> Self {
        Self {
            value: String::new(),
        }
    }

    pub fn into(self) -> String {
        self.value
    }
}

impl Output for StringOutput {
    fn write_str(&mut self, s: &str) -> Result<usize> {
        self.write(s.as_bytes())
    }
}

impl Write for StringOutput {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let s = match std::str::from_utf8(buf) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };
        self.value.push_str(s);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
