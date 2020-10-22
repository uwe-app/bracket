use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Syntax(#[from] SyntaxError),
    #[error(transparent)]
    Render(#[from] RenderError),
}

#[derive(Error, Debug)]
pub enum SyntaxError {
    #[error("Token parse error")]
    InvalidToken,
    #[error("Got an end block without a start block")]
    BadEndBlock,
    #[error("Raw block was not terminated")]
    RawBlockNotTerminated,
    #[error("Got an end block but no named block is open")]
    BadEndNamedBlock,
    #[error("Block {0} open but got closing block with name {1}")]
    BadBlockEndName(String, String),
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
