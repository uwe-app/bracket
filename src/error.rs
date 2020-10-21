use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Token parse error")]
    InvalidToken,
    #[error("Got an end block without a start block")]
    BadEndBlock,
    #[error("Got an end raw block but no raw block is open")]
    BadEndRawBlock,
    #[error("Got an end block but no named block is open")]
    BadEndNamedBlock,
    #[error("Block {0} open but got closing block with name {1}")]
    BadBlockEndName(String, String),
}

