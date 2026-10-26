use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown concept `{0}`")]
    UnknownConcept(String),

    #[error("a concept cannot be linked to itself (`{0}`)")]
    SelfLink(String),

    #[error("prerequisite `{from}` -> `{to}` would create a cycle")]
    PrerequisiteCycle { from: String, to: String },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
