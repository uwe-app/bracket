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
    EmptyStatement,
    #[error("Syntax error, expecting identifier")]
    ExpectedIdentifier,
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
