use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum Error {
    #[error(transparent)]
    Syntax(#[from] SyntaxError),
    #[error(transparent)]
    Render(#[from] RenderError),
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum SyntaxError {
    #[error("Syntax error, statement is empty")]
    EmptyStatement(usize),
    #[error("Syntax error, expecting identifier")]
    ExpectedIdentifier(usize),
}

impl SyntaxError {
    pub fn byte_offset(&self) -> &usize {
        match self {
            Self::EmptyStatement(pos) => pos,
            Self::ExpectedIdentifier(pos) => pos,
        }
    }

    pub fn to_source_context(&self, s: &str) -> String {
        let mut msg = String::new();
        msg
    }

    pub fn print(&self, s: &str) {
        eprintln!("{}", self.to_source_context(s));
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
